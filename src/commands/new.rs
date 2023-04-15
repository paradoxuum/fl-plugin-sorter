use clap::Parser;
use color_eyre::Result;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::config::{Config, PluginGroup, PluginGroupType};

use super::RunnableCommand;

/// Creates a new plugin group
#[derive(Debug, Parser)]
pub struct NewSubcommand {
    /// List of plugins the plugin group should contain
    #[arg(required = true)]
    plugins: Vec<String>,

    /// Name of the plugin group
    #[arg(long, short)]
    name: String,

    /// Type of the plugin group
    #[arg(long = "type", short = 't')]
    group_type: PluginGroupType,

    /// Name of the plugin group file
    #[arg(long, short)]
    file_name: Option<String>,
}

impl RunnableCommand for NewSubcommand {
    fn run(self, config: &Config) -> Result<()> {
        let group_data = match self.group_type {
            PluginGroupType::Effect => &config.effects,
            PluginGroupType::Generator => &config.generators,
        };

        if group_data.group_exists(&self.name) {
            let overwrite = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("That plugin group already exists. Do you want to overwrite it?")
                .interact()?;

            if !overwrite {
                return Ok(());
            }
        }

        let file_name = self
            .file_name
            .unwrap_or_else(|| self.name.to_lowercase().replace(' ', "_"));

        group_data.save_group(&file_name, &PluginGroup::new(&self.name, self.plugins))
    }
}
