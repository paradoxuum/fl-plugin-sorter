use std::collections::HashMap;

use clap::Parser;
use color_eyre::{eyre::eyre, Result};
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use owo_colors::OwoColorize;

use crate::config::{Config, PluginGroup};

use super::RunnableCommand;

#[derive(Debug, Parser)]
pub struct ListSubcommand {}

impl RunnableCommand for ListSubcommand {
    fn run(self, config: &Config) -> Result<()> {
        let mut groups = HashMap::<String, &PluginGroup>::new();
        for group in &config.effects.groups {
            groups.insert(format!("{} (EFFECT)", group.name), group);
        }

        for group in &config.generators.groups {
            groups.insert(format!("{} (GENERATOR)", group.name), group);
        }

        if groups.is_empty() {
            println!("{}", "There are no plugin groups defined".bright_red());
            return Ok(());
        }

        let mut names: Vec<String> = groups.keys().cloned().collect();
        names.sort();

        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a plugin group, type to search")
            .items(&names)
            .max_length(5)
            .interact_opt()?;

        if selection.is_none() {
            return Ok(());
        }

        if let Some(key) = names.get(selection.unwrap()) {
            let plugin_group = groups
                .get(key)
                .ok_or_else(|| eyre!("no plugin group with that name exists"))?;

            let mut plugin_text = String::new();
            for plugin in &plugin_group.plugins {
                plugin_text.push_str(format!("{}\n", plugin.green()).as_str());
            }

            println!(
                "{}\n\n{}\n{}",
                plugin_group.name.cyan().bold(),
                "Plugins".blue().bold().underline(),
                plugin_text
            );
        }
        Ok(())
    }
}
