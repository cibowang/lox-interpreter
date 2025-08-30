//use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use codecrafters_interpreter::*;
use miette::{self, Context, IntoDiagnostic};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Tokenize { filename: PathBuf },
}

fn main() -> miette::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Tokenize { filename } => {
            let file_contents = fs::read_to_string(&filename)
                .into_diagnostic()
                .wrap_err_with(|| format!("Reading '{}' failed", filename.display()))?; //.with_context(|| format!("reading '{:?}' failed", filename))?;;

            for token in Lexer::new(&file_contents) {
                let token = token?;
                println!("{token}");
            }
        }
    }
    Ok(())
}
