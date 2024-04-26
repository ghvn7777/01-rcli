use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use enum_dispatch::enum_dispatch;
use jsonwebtoken::Algorithm;
use tokio::fs;
use tracing::info;

use crate::{process_jwt_secret_generate, CmdExector};

use super::{parse_duration, verify_file, verify_path};

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum JwtSubCommand {
    #[command(about = "Sign a message with a private/session key")]
    Sign(JwtSignOpts),

    #[command(about = "Verify a signature with a public/session key")]
    Verify(JwtVerifyOpts),

    #[command(about = "Generate a random secret")]
    Generate(JwtGenerateOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,

    #[arg(short, long, default_value = "HS256")]
    pub algorithm: Algorithm,

    #[arg(long, default_value = "Kaka")]
    pub iss: String,

    #[arg(long, default_value = "14d", value_parser = parse_duration)]
    pub exp: Duration,

    #[arg(long)]
    pub sub: String,

    #[arg(long)]
    pub aud: String,
}

// todo: implement jwt verify
#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,

    #[arg(long)]
    pub token: String,
}

#[derive(Debug, Parser)]
pub struct JwtGenerateOpts {
    #[arg(short, long, value_parser = verify_path)]
    pub output_path: PathBuf,
}

impl CmdExector for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let mut reader = crate::get_reader(&self.key)?;

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)?;

        let exp = (since_the_epoch + self.exp).as_secs();

        let token = crate::process_jwt_sign(
            &mut reader,
            self.algorithm,
            self.iss,
            exp,
            self.sub,
            self.aud,
        )?;
        println!("token: {}", token);
        Ok(())
    }
}

impl CmdExector for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let mut reader = crate::get_reader(&self.key)?;
        info!("Verifying jwt: {:?}", self);
        if crate::process_jwt_verify(&mut reader, self.token)? {
            info!("Token is valid");
        } else {
            info!("Token is invalid");
        }

        Ok(())
    }
}

impl CmdExector for JwtGenerateOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let key = process_jwt_secret_generate()?;
        for (k, v) in key {
            fs::write(self.output_path.join(k), v).await?;
        }
        Ok(())
    }
}
