//! `glctl login` — browser-based authentication with glhub.
//!
//! Flow:
//!   1. Generate a random state token (CSRF guard)
//!   2. Bind a local HTTP server on a random port
//!   3. Open browser → https://glhub.baryon.ai/login/cli?redirect_uri=...&state=...
//!   4. Wait for the browser to redirect back → GET /callback?token=...&state=...
//!   5. Verify state, save token to ~/.glctl/config

use crate::config;
use crate::{CliError, CliResult};
use clap::Args;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

const CALLBACK_TIMEOUT_SECS: u64 = 300; // 5 minutes

#[derive(Args, Debug)]
pub struct LoginArgs {
    /// glhub base URL (overrides GLHUB_URL).
    #[arg(long)]
    pub remote: Option<String>,
}

pub fn run(args: LoginArgs) -> CliResult<()> {
    let base = args
        .remote
        .or_else(|| std::env::var("GLHUB_URL").ok())
        .unwrap_or_else(|| "https://glhub.baryon.ai".to_string());
    let base = base.trim_end_matches('/');

    let state = Uuid::new_v4().to_string();

    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| CliError::Error(format!("cannot start local callback server: {}", e)))?;
    let port = listener
        .local_addr()
        .map_err(|e| CliError::Error(format!("cannot read local addr: {}", e)))?
        .port();

    let redirect_uri = format!("http://localhost:{}/callback", port);
    let login_url = format!(
        "{}/login/cli?redirect_uri={}&state={}",
        base,
        percent_encode(&redirect_uri),
        state
    );

    eprintln!("Opening browser to authenticate with glhub...");
    if let Err(e) = open_browser(&login_url) {
        eprintln!("  (could not open browser automatically: {})", e);
    }
    eprintln!();
    eprintln!("If the browser did not open, visit:");
    eprintln!("  {}", login_url);
    eprintln!();
    eprintln!("Waiting for authentication (timeout: 5 min)...");

    let token = wait_for_callback(listener, &state)?;

    let mut cfg = config::load()?;
    cfg.token = Some(token.clone());
    config::save(&cfg)?;

    let path = config::config_path()?;
    println!();
    println!("Logged in successfully.");
    println!("Token saved → {}  ({})", mask(&token), path.display());
    Ok(())
}

// ── local callback server ──────────────────────────────────────────────────

fn wait_for_callback(listener: TcpListener, expected_state: &str) -> CliResult<String> {
    listener
        .set_nonblocking(true)
        .map_err(|e| CliError::Error(format!("set_nonblocking: {}", e)))?;

    let deadline = std::time::Instant::now() + Duration::from_secs(CALLBACK_TIMEOUT_SECS);

    loop {
        if std::time::Instant::now() > deadline {
            return Err(CliError::Error(
                "login timed out — no browser callback received".to_string(),
            ));
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                match handle_request(&mut stream, expected_state) {
                    Ok(Some(token)) => return Ok(token),
                    Ok(None) => {}  // wrong path or state mismatch; keep waiting
                    Err(e) => eprintln!("  callback error: {}", e),
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(CliError::Error(format!("accept error: {}", e))),
        }
    }
}

fn handle_request(stream: &mut TcpStream, expected_state: &str) -> CliResult<Option<String>> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| CliError::Error(format!("set_read_timeout: {}", e)))?;

    let mut buf = [0u8; 8192];
    let n = stream
        .read(&mut buf)
        .map_err(|e| CliError::Error(format!("read: {}", e)))?;
    let request = std::str::from_utf8(&buf[..n]).unwrap_or("");

    // first line: GET /callback?...  HTTP/1.1
    let first_line = request.lines().next().unwrap_or("");
    let raw_path = first_line.split_whitespace().nth(1).unwrap_or("");

    let (path, query) = match raw_path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (raw_path, ""),
    };

    if path != "/callback" {
        respond(stream, 404, "Not Found", "");
        return Ok(None);
    }

    let params = parse_query(query);
    let token = params.get("token").cloned();
    let state = params.get("state").cloned();

    if state.as_deref() != Some(expected_state) {
        respond(
            stream,
            400,
            "Bad Request",
            "<h2>Authentication failed</h2><p>State mismatch — please try again.</p>",
        );
        return Ok(None);
    }

    match token {
        Some(tok) if !tok.is_empty() => {
            respond(
                stream,
                200,
                "OK",
                "<h2>Authentication successful!</h2><p>You can close this tab and return to your terminal.</p>",
            );
            Ok(Some(tok))
        }
        _ => {
            respond(
                stream,
                400,
                "Bad Request",
                "<h2>Authentication failed</h2><p>No token received — please try again.</p>",
            );
            Ok(None)
        }
    }
}

fn respond(stream: &mut TcpStream, code: u16, reason: &str, html_body: &str) {
    let body = format!(
        "<!DOCTYPE html><html><head><meta charset=utf-8>\
         <title>glctl</title>\
         <style>body{{font-family:sans-serif;max-width:480px;margin:4rem auto;padding:0 1rem}}</style>\
         </head><body>{}</body></html>",
        html_body
    );
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        code,
        reason,
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

// ── helpers ────────────────────────────────────────────────────────────────

fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(percent_decode(k), percent_decode(v));
        }
    }
    map
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("unsupported platform — open the URL manually".to_string())
    }
}

fn mask(token: &str) -> String {
    if token.len() <= 8 {
        return "****".to_string();
    }
    format!("{}…{}", &token[..8], "****")
}
