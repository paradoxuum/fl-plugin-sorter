use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use owo_colors::OwoColorize;

use crate::{
    config::{Config, PluginGroup, PluginGroupType},
    plugin::is_path_vst,
};

use super::RunnableCommand;

/// Generates a plugin group from a folder containing VST files
#[derive(Debug, Parser)]
pub struct GenerateSubcommand {
    /// Path to a folder containing VST files
    path: PathBuf,

    /// Name of the plugin group to generate
    #[arg(long, short)]
    name: Option<String>,

    /// Name of the file to save generated plugin groups to
    #[arg(long, short)]
    file_name: Option<String>,

    /// Whether to include all plugins in subdirectories in the plugin group
    #[arg(long, action)]
    recurse: bool,
}

impl RunnableCommand for GenerateSubcommand {
    fn run(self, config: &Config) -> Result<()> {
        let mut plugin_names = Vec::<String>::new();
        self.get_plugin_names(&self.path, &mut plugin_names)?;

        let plugin_count = plugin_names.len();
        if plugin_count == 0 {
            return Err(eyre!("no plugins found in folder"));
        }

        let dir_name = self
            .path
            .file_name()
            .ok_or_else(|| eyre!("failed to get directory file name"))?
            .to_str()
            .ok_or_else(|| eyre!("failed to convert directory file name into string"))?;

        let group_name = self.name.clone().unwrap_or_else(|| dir_name.into());
        let file_name = self
            .file_name
            .clone()
            .unwrap_or_else(|| dir_name.to_lowercase().replace(' ', "_"));

        // Prompt the user to select effect plugins, the non-selected plugins are generator plugins
        let chosen = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select the plugins that are effects (SPACE: select, A: select all, ENTER: confirm)")
            .items(&plugin_names)
            .interact()?;

        // If none are selected, all of the plugins are generators, so the below code to separate the selected
        // items from the non-selected items can be skipped
        if chosen.is_empty() {
            let plugin_count = plugin_names.len();
            self.save_group(
                config,
                PluginGroupType::Generator,
                &file_name,
                &PluginGroup::new(&group_name, plugin_names),
            )?;
            self.display_saved_count(&file_name, PluginGroupType::Generator, plugin_count);
            return Ok(());
        }

        // Collect effects and generators into vectors based on which ones are chosen
        let chosen_indexes: HashSet<usize> = HashSet::from_iter(chosen.into_iter());
        let chosen_count = chosen_indexes.len();
        let mut effects = Vec::with_capacity(chosen_count);
        let mut generators = Vec::with_capacity(plugin_names.len() - chosen_count);
        for (i, plugin) in plugin_names.into_iter().enumerate() {
            if chosen_indexes.contains(&i) {
                effects.push(plugin);
            } else {
                generators.push(plugin);
            }
        }

        // Save effect and generator groups to files
        let effect_count = effects.len();
        let generator_count = generators.len();

        if effect_count > 0 {
            self.save_group(
                config,
                PluginGroupType::Effect,
                &file_name,
                &PluginGroup::new(&group_name, effects),
            )?;
            self.display_saved_count(&file_name, PluginGroupType::Effect, effect_count);
        }

        if generator_count > 0 {
            self.save_group(
                config,
                PluginGroupType::Generator,
                &file_name,
                &PluginGroup::new(&group_name, generators),
            )?;
            self.display_saved_count(&file_name, PluginGroupType::Generator, generator_count);
        }

        Ok(())
    }
}

impl GenerateSubcommand {
    fn save_group(
        &self,
        config: &Config,
        group_type: PluginGroupType,
        file_name: &str,
        plugin_group: &PluginGroup,
    ) -> Result<()> {
        let group = match group_type {
            PluginGroupType::Effect => &config.effects,
            PluginGroupType::Generator => &config.generators,
        };

        group
            .save_group(file_name, plugin_group)
            .wrap_err_with(|| eyre!("failed to save {} plugin group", group_type.name()))
    }

    fn get_plugin_names(&self, dir: &Path, plugin_names: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && self.recurse {
                self.get_plugin_names(&path, plugin_names)?;
                continue;
            }

            if !is_path_vst(&path) {
                continue;
            }

            plugin_names.push(
                path.file_stem()
                    .ok_or_else(|| eyre!("failed to get file name of plugin"))?
                    .to_str()
                    .ok_or_else(|| eyre!("failed to convert file name of plugin to string"))?
                    .to_owned(),
            )
        }

        Ok(())
    }

    fn display_saved_count(
        &self,
        file_name: &str,
        plugin_type: PluginGroupType,
        plugin_count: usize,
    ) {
        let plugin_type_name = match plugin_type {
            PluginGroupType::Effect => "effect",
            PluginGroupType::Generator => "generator",
        };

        println!(
            "{} {} {} {}",
            "Saved".green(),
            plugin_count.cyan().bold(),
            format!(
                "{} plugin{} to",
                plugin_type_name,
                if plugin_count == 1 { "" } else { "s" }
            )
            .green(),
            format!("{file_name}.toml").cyan().bold(),
        );
    }
}
