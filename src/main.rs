use rcli::{CmdExector, Opts};

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();
    opts.cmd.execute().await?;

    Ok(())
}
