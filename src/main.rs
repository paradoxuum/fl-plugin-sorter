use clap::Parser;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use commands::{RunnableCommand, Subcommand};
use config::Config;
use dirs::home_dir;

mod commands;
mod config;
mod plugin;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    // Load config
    let mut config_path = home_dir().ok_or_else(|| eyre!("failed to get home directory"))?;
    if !config_path.exists() {
        return Err(eyre!("home directory does not exist"));
    }

    config_path.push(".config/flsorter");
    let config = Config::from_file(&config_path).wrap_err("failed to load config")?;

    // Run subcommand
    match cli.subcommand {
        Subcommand::Generate(sub) => sub.run(&config),
        Subcommand::List(sub) => sub.run(&config),
        Subcommand::New(sub) => sub.run(&config),
        Subcommand::Sort(sub) => sub.run(&config),
        Subcommand::Unsort(sub) => sub.run(&config),
    }
}
