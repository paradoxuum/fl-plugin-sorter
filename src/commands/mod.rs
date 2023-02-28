use clap::Parser;
use color_eyre::Result;

use crate::config::Config;

use self::{
    generate::GenerateSubcommand, list::ListSubcommand, new::NewSubcommand, sort::SortSubcommand,
    unsort::UnsortSubcommand,
};

mod generate;
mod list;
mod new;
mod sort;
mod unsort;

#[derive(Debug, Parser)]
pub enum Subcommand {
    Generate(GenerateSubcommand),
    List(ListSubcommand),
    New(NewSubcommand),
    Sort(SortSubcommand),
    Unsort(UnsortSubcommand),
}

pub trait RunnableCommand {
    fn run(self, config: &Config) -> Result<()>;
}
