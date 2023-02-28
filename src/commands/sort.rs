use std::{fs, path::Path};

use clap::Parser;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use owo_colors::OwoColorize;

use crate::config::{Config, InstalledPlugins, PluginGroup};

use super::RunnableCommand;

struct SortResult {
    folder_count: u32,
    plugin_count: u32,
}

#[derive(Debug, Parser)]
/// Sorts plugins defined by plugin group files into the plugin database
pub struct SortSubcommand {}

impl RunnableCommand for SortSubcommand {
    fn run(self, config: &Config) -> Result<()> {
        let is_effects_empty = config.effects.groups.is_empty();
        let is_generators_empty = config.generators.groups.is_empty();
        if is_effects_empty && is_generators_empty {
            return Err(eyre!("there are no plugin groups to sort"));
        }

        let plugin_database = &config.plugin_database;
        if !is_effects_empty {
            self.display_result(
                self.sort_groups(
                    &plugin_database.effects.folder,
                    &plugin_database.effects.installed,
                    &config.effects.groups,
                )?,
                "effect",
            );
        }

        if !is_generators_empty {
            self.display_result(
                self.sort_groups(
                    &plugin_database.generators.folder,
                    &plugin_database.generators.installed,
                    &config.generators.groups,
                )?,
                "generator",
            );
        }

        Ok(())
    }
}

impl SortSubcommand {
    fn sort_groups(
        &self,
        plugin_folder: &Path,
        installed_plugins: &InstalledPlugins,
        groups: &Vec<PluginGroup>,
    ) -> Result<SortResult> {
        let mut result = SortResult {
            folder_count: 0,
            plugin_count: 0,
        };

        for group in groups {
            if group.plugins.is_empty() {
                println!(
                    "{}{}{}",
                    "Skipping '".green(),
                    group.name.cyan().bold(),
                    "' because no plugins are defined".green()
                );
                continue;
            }

            let group_dir = plugin_folder.join(&group.name);
            fs::create_dir_all(&group_dir).wrap_err("failed to create group directory")?;

            // Copy over plugins to group folder
            for plugin_name in &group.plugins {
                let plugin_path = installed_plugins.get_plugin(plugin_name);
                if let Some(path) = plugin_path {
                    let destination = group_dir.join(format!("{plugin_name}.fst"));
                    fs::copy(path, destination)
                        .wrap_err_with(|| format!("failed to copy '{plugin_name}'"))?;

                    result.plugin_count += 1;
                } else {
                    println!(
                        "{}{}{}",
                        "Skipping '".yellow(),
                        plugin_name.blue().bold(),
                        "' because it is not installed".yellow()
                    )
                }
            }

            result.folder_count += 1;
        }

        Ok(result)
    }

    fn display_result(&self, result: SortResult, plugin_type: &str) {
        println!(
            "{} {} {} {} {}",
            "Successfully sorted".green(),
            result.plugin_count.cyan().bold(),
            format!(
                "{} plugin{} into",
                plugin_type,
                if result.plugin_count == 1 { "" } else { "s" }
            )
            .green(),
            result.folder_count.cyan().bold(),
            format!("folder{}", if result.folder_count == 1 { "" } else { "s" }).green()
        );
    }
}
