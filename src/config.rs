use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use dirs::document_dir;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

/// Represents the different types of possible plugin groups
#[derive(ValueEnum, Clone, Debug)]
pub enum PluginGroupType {
    Effect,
    Generator,
}

impl PluginGroupType {
    /// Returns the name of the plugin group type
    pub fn name(&self) -> String {
        match self {
            Self::Effect => "effect",
            Self::Generator => "generator",
        }
        .to_owned()
    }

    /// Returns a [PathBuf] pointing to the plugin group type directory
    pub fn path(&self, base_path: &Path) -> PathBuf {
        base_path.join(self.name())
    }
}

/// A data structure that defines the name of a
/// group of plugins and a list containing the names
/// of plugins that should be sorted into that group.
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginGroup {
    pub name: String,
    pub plugins: Vec<String>,
}

impl PluginGroup {
    pub fn new(name: &str, plugins: Vec<String>) -> Self {
        Self {
            name: name.to_owned(),
            plugins,
        }
    }

    /// Creates a [`PluginGroup`] from a [`Path`] pointing to
    /// a TOML file through deserialization.
    fn from_file(path: &Path) -> Result<Self> {
        let file_name = path
            .file_name()
            .ok_or_else(|| eyre!("failed to get group file name"))?
            .to_str()
            .ok_or_else(|| eyre!("failed to convert group file name to string"))?;

        if !path.is_file() {
            return Err(eyre!("provided path is not a file"));
        }

        let contents = fs::read_to_string(path)
            .wrap_err_with(|| eyre!("failed to read contents of {}", file_name))?;

        toml::from_str(&contents).wrap_err_with(|| eyre!("failed to parse {}", file_name))
    }
}

/// A structure that groups together a
/// `PluginGroupType` with a [`Vec`] containing
/// `PluginGroup`s.
///
/// The [`Path`] to the folder containing the plugin
/// groups is also included as the `path` field.
#[derive(Debug)]
pub struct PluginGroupData {
    pub group_type: PluginGroupType,
    pub config_path: PathBuf,
    pub groups: Vec<PluginGroup>,
}

impl PluginGroupData {
    fn new(group_type: PluginGroupType, path: &Path, groups: Vec<PluginGroup>) -> Self {
        Self {
            group_type,
            config_path: path.to_owned(),
            groups,
        }
    }

    pub fn group_path(&self, group_name: &str) -> PathBuf {
        self.config_path.join(format!("{group_name}.toml"))
    }

    pub fn group_exists(&self, group_name: &str) -> bool {
        let path = self.group_path(group_name);
        path.exists() && path.is_file()
    }

    pub fn save_group(&self, file_name: &str, group: &PluginGroup) -> Result<()> {
        let file_path = self.group_path(file_name);
        let contents = toml::to_string(group)
            .wrap_err_with(|| eyre!("failed to convert '{}' into a TOML string", group.name))?;

        fs::write(file_path, contents)
            .wrap_err_with(|| eyre!("failed to write '{}' to a file", group.name))
    }
}

/// Contains two [`Path`]s pointing to a directory
/// containing installed plugin files.
///
/// These folders contain either `VST3` plugins or
/// `VST` plugins pointed to in `.fst` files.
#[derive(Debug)]
pub struct InstalledPlugins {
    pub vst: PathBuf,
    pub vst3: PathBuf,
}

impl InstalledPlugins {
    fn new(vst: &Path, vst3: &Path) -> Self {
        Self {
            vst: vst.to_owned(),
            vst3: vst3.to_owned(),
        }
    }

    /// Creates a `InstalledPlugins` from a [`Path`].
    ///
    /// # Errors
    /// The function will return an error if the given [`Path`] does not contain
    /// a `VST` or `VST3` subdirectory.
    fn from_folder(plugin_folder: &Path) -> Result<Self> {
        let vst3 = plugin_folder.join("VST3");
        let vst = plugin_folder.join("VST");

        if (!vst3.exists() || !vst.exists()) || (!vst3.is_dir() || !vst.is_dir()) {
            return Err(eyre!(
                "installed plugins folder does not contain VST or VST3 folders"
            ));
        }

        Ok(Self::new(&vst, &vst3))
    }

    pub fn get_plugin(&self, name: &str) -> Option<PathBuf> {
        let file_name = format!("{name}.fst");
        let vst3 = self.vst3.join(&file_name);
        if vst3.exists() {
            return Some(vst3);
        }

        let vst = self.vst.join(&file_name);
        if vst.exists() {
            return Some(vst);
        }

        None
    }
}

/// A data structure containing the installed plugins,
/// and folder containing sorted plugins for a `PluginGroupType`.
#[derive(Debug)]
pub struct PluginDatabaseGroup {
    pub group_type: PluginGroupType,
    pub installed: InstalledPlugins,
    pub folder: PathBuf,
}

impl PluginDatabaseGroup {
    fn new(group_type: PluginGroupType, installed: InstalledPlugins, folder: &Path) -> Self {
        Self {
            group_type,
            installed,
            folder: folder.to_owned(),
        }
    }
}

/// Represents the plugin database structure that is used by Fl Studio.
#[derive(Debug)]
pub struct PluginDatabase {
    pub effects: PluginDatabaseGroup,
    pub generators: PluginDatabaseGroup,
}

impl PluginDatabase {
    fn new(database_path: &Path) -> Result<Self> {
        // Check an array of paths, all paths must exist
        // to ensure it is valid
        let effects = database_path.join("Effects");
        let generators = database_path.join("Generators");
        let installed_plugins = database_path.join("Installed");
        let installed_effects = installed_plugins.join("Effects");
        let installed_generators = installed_plugins.join("Generators");
        let paths = [
            &effects,
            &generators,
            &installed_effects,
            &installed_generators,
        ];

        if !paths.into_iter().all(|p| p.exists()) {
            return Err(eyre!("plugin database structure is invalid"));
        }

        Ok(Self {
            effects: PluginDatabaseGroup::new(
                PluginGroupType::Effect,
                InstalledPlugins::from_folder(&installed_effects)?,
                &effects,
            ),
            generators: PluginDatabaseGroup::new(
                PluginGroupType::Generator,
                InstalledPlugins::from_folder(&installed_generators)?,
                &generators,
            ),
        })
    }

    pub fn get_group_path(&self, group: &PluginGroup, group_type: &PluginGroupType) -> PathBuf {
        let database_group = match group_type {
            PluginGroupType::Effect => &self.effects,
            PluginGroupType::Generator => &self.generators,
        };

        database_group.folder.join(&group.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    pub plugin_database_path: PathBuf,
}

impl UserConfig {
    pub fn new(config_dir: &Path) -> Result<Self> {
        // Ensure config file exists and is valid
        let config_file = config_dir.join("config.toml");
        if config_file.exists() {
            let contents =
                fs::read_to_string(&config_file).wrap_err("failed to read config.toml")?;
            let config: Self = toml::from_str(&contents).wrap_err("failed to parse config.toml")?;
            return Ok(config);
        }

        // Create and write default configuration to file
        let mut plugin_database_path =
            document_dir().ok_or_else(|| eyre!("failed to get 'Documents' directory"))?;
        plugin_database_path.push("Image-Line/FL Studio/Presets/Plugin database");

        let config = Self {
            plugin_database_path,
        };

        let contents = toml::to_string(&config).wrap_err("failed to serialize user config")?;
        fs::write(config_file, contents).wrap_err("failed to write config.toml")?;

        Ok(config)
    }
}

#[derive(Debug)]
pub struct Config {
    pub user: UserConfig,
    pub plugin_database: PluginDatabase,
    pub effects: PluginGroupData,
    pub generators: PluginGroupData,
}

impl Config {
    fn new(
        user_config: UserConfig,
        plugin_database: PluginDatabase,
        effects: PluginGroupData,
        generators: PluginGroupData,
    ) -> Self {
        Self {
            user: user_config,
            plugin_database,
            effects,
            generators,
        }
    }

    /// Creates a new `Config` from the given [`Path`].
    pub fn from_file(config_path: &Path) -> Result<Self> {
        // Create config directories if they don't exist
        Self::create_directory(config_path)?;

        // Create user config and plugin database
        let user_config = UserConfig::new(config_path)?;
        let plugin_database = PluginDatabase::new(&user_config.plugin_database_path)?;

        // Get directories containing plugin group definitions and create them if they don't exist
        let effects_dir = PluginGroupType::Effect.path(config_path);
        let generators_dir = PluginGroupType::Generator.path(config_path);
        Self::create_directory(&effects_dir)?;
        Self::create_directory(&generators_dir)?;

        // Get plugin groups
        let effects = PluginGroupData::new(
            PluginGroupType::Effect,
            &effects_dir,
            Self::groups(&effects_dir)?,
        );

        let generators = PluginGroupData::new(
            PluginGroupType::Generator,
            &generators_dir,
            Self::groups(&generators_dir)?,
        );

        Ok(Self::new(user_config, plugin_database, effects, generators))
    }

    /// Creates a [`Vec`] of any `PluginGroup`s found in the given [`Path`]
    /// by deserializing any TOML files in the directory.
    fn groups(path: &Path) -> Result<Vec<PluginGroup>> {
        let mut groups = Vec::new();
        let mut group_names = HashSet::<String>::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            // Ensure path is a file and extension is correct before attempting to deserialize
            let extension = path.extension();
            if !path.is_file() || (extension.is_none() || extension.unwrap() != "toml") {
                continue;
            }

            let group = PluginGroup::from_file(&path)?;
            if group_names.contains(&group.name) {
                println!(
                    "{}{}{}{}",
                    "WARN: A plugin group with the name '".yellow(),
                    group.name.blue(),
                    "' already exists. Overwriting with the group defined in ".yellow(),
                    path.file_name()
                        .ok_or_else(|| eyre!("failed to unwrap OsStr for {}", path.display()))?
                        .to_str()
                        .ok_or_else(|| eyre!("failed to unwrap str for {}", path.display()))?
                        .blue()
                );
            } else {
                group_names.insert(group.name.to_owned());
            }

            groups.push(group);
        }

        Ok(groups)
    }

    /// Creates a directory, **including parents**, from the given [`Path`].
    ///
    /// Any [`Err`] returned is wrapped with a more human readable message,
    /// stating which [`Path`] failed to be created.
    ///
    /// # Errors
    /// This function inherits the same errors from [`fs::create_dir_all`].
    fn create_directory(path: &Path) -> Result<()> {
        fs::create_dir_all(path).wrap_err(format!("failed to create {}", path.display()))
    }
}
