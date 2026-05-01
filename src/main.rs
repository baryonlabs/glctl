//! glctl — Generation Lineage Control Tool
//!
//! AI 에이전트 실행의 generation 계보를 YAML 파일로 추적·관리한다.
//!
//! Exit codes:
//!   0 = 성공
//!   1 = 데이터 없음 (generations/ 비어있음 등)
//!   2 = 에러 (I/O, 파싱 등)

mod commands;
mod config;
mod models;
mod storage;

use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "glctl",
    version,
    about = "Generation Lineage Control Tool",
    long_about = "AI 에이전트 실행의 generation 계보를 YAML 파일로 추적·관리하는 CLI."
)]
struct Cli {
    /// Paperclip company id used to scope storage. Can also be set with GLCTL_COMPANY_ID.
    #[arg(long = "company-id", global = true)]
    company_id: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Authenticate with glhub interactively via browser.
    Login(commands::login::LoginArgs),

    /// Authenticate with glhub by saving a Personal Access Token directly.
    Auth(commands::auth::AuthArgs),

    /// Initialize the company-scoped generation repository.
    Init(commands::init::InitArgs),

    /// Create a new generation record.
    New(commands::new::NewArgs),

    /// Show one generation record.
    Show(commands::show::ShowArgs),

    /// Output the lineage (nodes + edges).
    Lineage(commands::lineage::LineageArgs),

    /// Output Mermaid flowchart DSL for the lineage.
    Graph(commands::graph::GraphArgs),

    /// List generations in reverse-chronological order.
    List(commands::list::ListArgs),

    /// Summarize the current company-scoped repository.
    Status(commands::status::StatusArgs),

    /// Check repository integrity.
    Fsck(commands::fsck::FsckArgs),

    /// Push this company-scoped lineage snapshot to glhub.
    Push(commands::push::PushArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Some(company_id) = cli.company_id.as_deref() {
        std::env::set_var("GLCTL_COMPANY_ID", company_id);
    }

    let result = match cli.command {
        Command::Login(args) => commands::login::run(args),
        Command::Auth(args) => commands::auth::run(args),
        Command::Init(args) => commands::init::run(args),
        Command::New(args) => commands::new::run(args),
        Command::Show(args) => commands::show::run(args),
        Command::Lineage(args) => commands::lineage::run(args),
        Command::Graph(args) => commands::graph::run(args),
        Command::List(args) => commands::list::run(args),
        Command::Status(args) => commands::status::run(args),
        Command::Fsck(args) => commands::fsck::run(args),
        Command::Push(args) => commands::push::run(args),
    };

    match result {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            // exit code is encoded in the CliError variant
            let code = e.exit_code();
            eprintln!("glctl: {}", e);
            ExitCode::from(code)
        }
    }
}

/// 공용 에러 타입. exit code를 자체적으로 제공.
#[derive(Debug)]
pub enum CliError {
    /// exit code 1
    NoData(String),
    /// exit code 2
    Error(String),
}

impl CliError {
    fn exit_code(&self) -> u8 {
        match self {
            CliError::NoData(_) => 1,
            CliError::Error(_) => 2,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::NoData(msg) => write!(f, "{}", msg),
            CliError::Error(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Error(format!("io error: {}", e))
    }
}

impl From<serde_yaml::Error> for CliError {
    fn from(e: serde_yaml::Error) -> Self {
        CliError::Error(format!("yaml error: {}", e))
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Error(format!("json error: {}", e))
    }
}

pub type CliResult<T> = Result<T, CliError>;
