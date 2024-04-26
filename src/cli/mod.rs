mod base64;
mod csv;
mod genpass;
mod http;
mod jwt;
mod text;

use std::path::{Path, PathBuf};

use clap::Parser;
use enum_dispatch::enum_dispatch;

pub use self::{base64::*, csv::*, genpass::*, http::*, jwt::*, text::*};

#[derive(Debug, Parser)]
#[clap(name = "rcli", version, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
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

    #[command(subcommand, about = "JWT sign/verify")]
    Jwt(JwtSubCommand),
}

fn parse_duration(duration_str: &str) -> Result<std::time::Duration, &'static str> {
    let str_len = duration_str.len();
    if str_len < 2 {
        return Err("Invalid duration");
    }
    let duration_unit = &duration_str[str_len - 1..];
    let duration = &duration_str[..str_len - 1];

    let mul_unit = match duration_unit {
        "s" => 1,
        "m" => 60,
        "h" => 60 * 60,
        "d" => 60 * 60 * 24,
        "w" => 60 * 60 * 24 * 7,
        "M" => 60 * 60 * 24 * 30,
        "y" => 60 * 60 * 24 * 365,
        _ => return Err("Invalid duration unit"),
    };

    let duration = duration.parse::<u64>().map_err(|_| "Invalid duration")?;
    Ok(std::time::Duration::from_secs(duration * mul_unit))
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
