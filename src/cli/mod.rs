mod base64;
mod csv;
mod genpass;
mod http;
mod text;

use std::path::{Path, PathBuf};

use crate::CmdExector;

use self::{csv::CsvOpts, genpass::GenPassOpts};
use clap::Parser;

pub use self::{
    base64::{Base64Format, Base64SubCommand},
    csv::OutputFormat,
    http::HttpSubCommand,
    text::{TextSignFormat, TextSubCommand},
};

#[derive(Debug, Parser)]
#[clap(name = "rcli", version, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    #[command(
        name = "csv",
        version,
        about = "Show CSV, or convert CSV to other formats"
    )]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),

    #[command(subcommand, about = "Base64 encode/decode")]
    Base64(Base64SubCommand),

    #[command(subcommand, about = "Text sign/verify")]
    Text(TextSubCommand),

    #[command(subcommand, about = "HTTP server")]
    Http(HttpSubCommand),
}

impl CmdExector for SubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            SubCommand::Csv(opts) => opts.execute().await,
            SubCommand::Base64(opts) => opts.execute().await,
            SubCommand::GenPass(opts) => opts.execute().await,
            SubCommand::Http(opts) => opts.execute().await,
            SubCommand::Text(opts) => opts.execute().await,
        }
    }
}

fn verify_file(filename: &str) -> Result<String, &'static str> {
    // if input is "-" or file exists
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("File does not exits")
    }
}

fn verify_path(path: &str) -> Result<PathBuf, &'static str> {
    // if input is "-" or file exists
    let p = Path::new(path);
    if p.exists() && p.is_dir() {
        Ok(path.into())
    } else {
        Err("Path dose not exist or is not a directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_file("-"), Ok("-".into()));
        assert_eq!(verify_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(verify_file("not-exists"), Err("File does not exits"));
        assert_eq!(verify_file("*"), Err("File does not exits"));
    }
}
