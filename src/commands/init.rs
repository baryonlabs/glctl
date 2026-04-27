//! `glctl init` — initialize company-scoped storage.

use crate::storage;
use crate::CliResult;
use clap::Args;
use serde::Serialize;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// JSON object output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct InitOutput {
    company_id: String,
    data_dir: String,
    company_dir: String,
    generations_dir: String,
    relations_dir: String,
}

pub fn run(args: InitArgs) -> CliResult<()> {
    storage::ensure_dirs()?;

    let out = InitOutput {
        company_id: storage::company_id()?,
        data_dir: storage::data_dir().display().to_string(),
        company_dir: storage::company_dir()?.display().to_string(),
        generations_dir: storage::generations_dir()?.display().to_string(),
        relations_dir: storage::relations_dir()?.display().to_string(),
    };

    if args.json {
        println!("{}", serde_json::to_string(&out)?);
    } else {
        println!("initialized {}", out.company_dir);
    }
    Ok(())
}
