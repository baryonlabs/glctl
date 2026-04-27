//! `glctl show <ID> [--json]` — show one generation.

use crate::storage;
use crate::CliResult;
use clap::Args;

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Generation id to show.
    pub id: String,

    /// JSON object output. Without this flag, prints the canonical YAML.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: ShowArgs) -> CliResult<()> {
    let gen = storage::load_generation_by_id(&args.id)?;
    if args.json {
        println!("{}", serde_json::to_string(&gen)?);
    } else {
        println!("{}", serde_yaml::to_string(&gen)?);
    }
    Ok(())
}
