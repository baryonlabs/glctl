//! `glctl auth` — save a glhub Personal Access Token to ~/.glctl/config.

use crate::config;
use crate::{CliError, CliResult};
use clap::Args;
use std::io::{self, BufRead, Write};

#[derive(Args, Debug)]
pub struct AuthArgs {
    /// glhub Personal Access Token. If omitted, prompted interactively.
    #[arg(long)]
    pub token: Option<String>,

    /// Print the currently stored token status and exit.
    #[arg(long)]
    pub status: bool,
}

pub fn run(args: AuthArgs) -> CliResult<()> {
    if args.status {
        let cfg = config::load()?;
        match cfg.token {
            Some(tok) => {
                let path = config::config_path()?;
                println!("Logged in — token: {}  ({})", mask(&tok), path.display());
            }
            None => println!("Not logged in. Run `glctl auth --token <TOKEN>` to authenticate."),
        }
        return Ok(());
    }

    let token = match args.token {
        Some(t) => t,
        None => {
            eprint!("glhub Personal Access Token: ");
            io::stderr()
                .flush()
                .map_err(|e| CliError::Error(format!("flush: {}", e)))?;
            let mut line = String::new();
            io::stdin()
                .lock()
                .read_line(&mut line)
                .map_err(|e| CliError::Error(format!("read: {}", e)))?;
            line
        }
    };

    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(CliError::Error("token cannot be empty".to_string()));
    }

    let mut cfg = config::load()?;
    cfg.token = Some(token.clone());
    config::save(&cfg)?;

    let path = config::config_path()?;
    println!("Token saved → {}  ({})", mask(&token), path.display());
    Ok(())
}

fn mask(token: &str) -> String {
    if token.len() <= 8 {
        return "****".to_string();
    }
    format!("{}…{}", &token[..8], "****")
}
