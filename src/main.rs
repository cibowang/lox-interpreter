use clap::{Parser, Subcommand};
// import module
use codecrafters_interpreter as imp;
use miette::{Context, IntoDiagnostic};
use std::fs;
use std::path::PathBuf;

// DIY cmd
#[derive(Debug, Subcommand)]
enum Commands {
    Tokenize { filepath: PathBuf },
    Parse { filepath: PathBuf },
    Run { filepath: PathBuf },
}

// define subcmd/args for parsing
// pattern match on DIY cmd..
#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

fn main() -> miette::Result<()> {
    // create cmd
    let args = Args::parse();
    match args.cmd {
        Commands::Tokenize { filepath } => {
            let file = fs::read_to_string(&filepath)
                .into_diagnostic()
                .wrap_err_with(|| format!("reading '{}' failed", filepath.display()))?;
            for token in imp::Lexer::new(&file) {
                let token = match token {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("{e:?}");
                        if let Some(unrecognized) = e.downcast_ref::<imp::lex::SingleTokenError>() {
                            eprintln!(
                                "[line {}] Error: unexpectcted char {}",
                                unrecognized.line(),
                                unrecognized
                            );
                        } else if let Some(unterminated) =
                            e.downcast_ref::<imp::lex::StringTerminationError>()
                        {
                            eprintln!(
                                "[line {}] Error: unexpectcted char {}",
                                unterminated.line(),
                                unterminated
                            );
                        }
                        // if wrong, dont stop
                        continue;
                    }
                };
                // return from loop
                println!("{token}");
            }
            // otherwise return from arm
            println!("Eof null");
        }
        Commands::Parse { filepath } => {
            let file = fs::read_to_string(&filepath)
                .into_diagnostic()
                .wrap_err_with(|| format!("reading '{}' failed", filepath.display()))?;
            let parser = imp::Parser::new(&file);
            match parser.parse() {
                Ok(tt) => tt,
                Err(e) => {
                    eprintln!("{e:?}");
                    std::process::exit(65);
                }
            };
        }
        Commands::Run { filepath } => {
            let file = fs::read_to_string(&filepath)
                .into_diagnostic()
                .wrap_err_with(|| format!("reading '{}' failed", filepath.display()))?;
            let parser = imp::Parser::new(&file);
            println!("{}", parser.parse().unwrap());
        }
    };
    Ok(())
}
