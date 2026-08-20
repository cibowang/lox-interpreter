use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

enum Commands {
    Tokenize { filename: PathBuf },
    Parse { filename: PathBuf },
    Run { filename: PathBuf },
}
