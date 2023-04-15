use std::fs;

use clap::Parser;
use color_eyre::Result;
use owo_colors::OwoColorize;

use crate::config::{Config, PluginDatabase, PluginGroup, PluginGroupType};

use super::RunnableCommand;

/// Removes any folders and plugin files created when sorting
#[derive(Debug, Parser)]
pub struct UnsortSubcommand {}

impl RunnableCommand for UnsortSubcommand {
    fn run(self, config: &Config) -> Result<()> {
        let plugin_database = &config.plugin_database;
        self.remove_sorted_files(
            plugin_database,
            &config.effects.groups,
            PluginGroupType::Effect,
        )?;

        self.remove_sorted_files(
            plugin_database,
            &config.generators.groups,
            PluginGroupType::Generator,
        )?;

        Ok(())
    }
}

impl UnsortSubcommand {
    /// Removes files from the plugin database.
    /// The type of plugin removed is specified by `group_type`.
    ///
    /// The files removed depends on which files are sorted from the
    /// defined plugin groups.
    ///
    /// If a plugin group directory is empty after having its files deleted,
    /// it will also be deleted.
    fn remove_sorted_files(
        &self,
        plugin_database: &PluginDatabase,
        groups: &Vec<PluginGroup>,
        group_type: PluginGroupType,
    ) -> Result<()> {
        if groups.is_empty() {
            println!(
                "{} {}{}",
                "Skipped".green(),
                group_type.name().cyan().bold(),
                "s because there are no plugin groups.".green(),
            );
            return Ok(());
        }

        let mut removed_count = 0;
        for group in groups {
            let base_path = plugin_database.get_group_path(group, &group_type);
            if !base_path.exists() {
                continue;
            }

            // Remove plugin files
            for plugin in &group.plugins {
                let plugin_path = base_path.join(format!("{plugin}.fst"));
                if !plugin_path.exists() || !plugin_path.is_file() {
                    continue;
                }

                fs::remove_file(plugin_path)?;
                removed_count += 1;
            }

            // Remove directory if it is empty
            if fs::read_dir(&base_path)?.next().is_none() {
                fs::remove_dir(base_path)?;
            }
        }

        if removed_count == 0 {
            println!(
                "{} {} {}",
                "Found no".green(),
                group_type.name().cyan().bold(),
                "plugins to unsort.".green()
            );
        } else {
            println!(
                "{} {} {}",
                "Successfully unsorted".green(),
                removed_count.cyan().bold(),
                format!(
                    "{} plugin{}.",
                    group_type.name(),
                    if removed_count == 1 { "" } else { "s" }
                )
                .green()
            );
        }

        Ok(())
    }
}
