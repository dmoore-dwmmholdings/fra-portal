use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ntsb", about = "NTSB ingest pipeline (docs/NTSB_INGEST_SPEC.md)")]
struct Cli {
    /// Root data directory.
    #[arg(long, default_value = "data", global = true)]
    data_dir: PathBuf,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Download avall/Pre2008 MDBs and docs into data/raw/<date>/ and unzip.
    Fetch {
        /// Download even if the manifest says the file is unchanged.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Fetch { force } => {
            let fetched = ntsb_fetch::fetch(&ntsb_fetch::FetchOptions {
                raw_dir: cli.data_dir.join("raw"),
                force,
            })?;
            println!("fetched {} file(s)", fetched.len());
        }
    }
    Ok(())
}
