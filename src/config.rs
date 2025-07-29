//! This module handles the configuration I/O for the application.

use std::fmt;
use std::str::FromStr;

use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::CONFIG;
use crate::io::{does_config_exist, get_config_path, write_default_config};

/// Represents the global configuration for the application.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub lexer: LexerConfig,
    #[serde(default)]
    pub html: HtmlConfig,
}

/// Manages all configuration for tokenization
#[derive(Debug, Deserialize, Serialize)]
pub struct LexerConfig {
    #[serde(default = "default_tab_size")]
    pub tab_size: usize,
}

impl Default for LexerConfig {
    fn default() -> Self {
        LexerConfig { tab_size: 4 }
    }
}

fn default_tab_size() -> usize {
    4
}

/// Manages all configuration for HTML generation
#[derive(Debug, Deserialize, Serialize)]
pub struct HtmlConfig {
    #[serde(default = "default_css")]
    pub css_file: String,
    #[serde(default)]
    pub favicon_file: String,
    #[serde(default)]
    pub use_prism: bool,
    #[serde(default = "default_prism_theme")]
    pub prism_theme: String,
    #[serde(default = "sanitize_by_default")]
    pub sanitize_html: bool,
}

impl Default for HtmlConfig {
    fn default() -> Self {
        HtmlConfig {
            css_file: default_css(),
            favicon_file: "".to_string(),
            use_prism: false,
            prism_theme: default_prism_theme(),
            sanitize_html: sanitize_by_default(),
        }
    }
}

/// Sets the default PrismJS theme to "vsc-dark-plus" in `config.toml`
fn default_prism_theme() -> String {
    "vsc-dark-plus".to_string()
}

/// Sets `sanitize_html` to true by default in `config.toml`
fn sanitize_by_default() -> bool {
    true
}

/// Sets the default CSS file to "default" in the case that the `css_file` field is omitted
fn default_css() -> String {
    "default".to_string()
}

impl Config {
    /// Creates a new `Config` instance from the specified file path
    ///
    /// # Arguments
    /// * `file_path` - The path to the configuration file. If no file path is provided as a CLI
    ///   arg, it will check for a config file in the default config directory.
    ///
    /// # Returns
    /// Returns a `Result` containing the `Config` instance if successful
    pub fn from_file(file_path: &str) -> Result<Self, Error> {
        // If the user provided a config file, try to load the config from it
        if !file_path.is_empty() {
            info!("Loading config from file: {}", file_path);
            let contents = std::fs::read_to_string(file_path)?;

            let config: Config = toml_edit::de::from_str(&contents)?;

            validate_config(file_path, &contents, &config)?;

            return Ok(config);
        }

        let config_path = get_config_path()?;

        // If the user did not provide a config file, check if a config file exists in the config
        // directory
        if does_config_exist()? {
            let contents = std::fs::read_to_string(&config_path)?;

            let config: Config =
                toml_edit::de::from_str(&contents).map_err(Error::TomlDeserialization)?;

            validate_config(&config_path.to_string_lossy(), &contents, &config)?;

            Ok(config)
        } else {
            warn!(
                "No config file found, writing default config to: {}",
                config_path.to_string_lossy()
            );

            let default_config = write_default_config()?;

            Ok(default_config)
        }
    }
}

/// Validates the configuration by checking if the original config file matches the filled config
///
/// If the original config is missing fields, it updates the file with any missing fields
fn validate_config(file_path: &str, contents: &str, config: &Config) -> Result<(), Error> {
    let mut doc = toml_edit::DocumentMut::from_str(contents).map_err(Error::Toml)?;

    let filled_doc = toml_edit::ser::to_document(config)?;

    let mut config_needs_update = false;
    let mut missing_fields = Vec::new();
    for (section, values) in filled_doc.iter() {
        let table = values.clone().into_table().unwrap_or_else(|_item| {
            error!(
                "Expected a table for field '{}', but found: {}",
                section, values
            );
            panic!("Invalid configuration format for field '{}'", section);
        });

        for (sub_key, sub_value) in table.iter() {
            if !doc.contains_key(section) {
                doc[section] = filled_doc[section].clone();
                config_needs_update = true;
                missing_fields.push(section.to_string());
            } else if !doc[section].is_table()
                || !doc[section].as_table().unwrap().contains_key(sub_key)
            {
                doc[section][sub_key] = sub_value.clone();
                config_needs_update = true;
                missing_fields.push(format!("{}.{}", section, sub_key));
            }
        }
    }

    if config_needs_update {
        warn!(
            "Config is missing fields: {:?}, writing updated config to: {}",
            missing_fields, file_path
        );

        // Formats the file with sections like `[lexer]` and `tab_size = 4`
        // previously it would be `lexer = { tab_size = 4 }`
        if !doc["lexer"].is_table() {
            doc["lexer"] = doc["lexer"]
                .clone()
                .into_table()
                .unwrap_or_else(|_item| {
                    error!(
                        "Expected 'lexer' to be a table, but found: {}",
                        doc["lexer"]
                    );
                    panic!("Invalid configuration format for 'lexer'");
                })
                .into();
        }
        doc["lexer"].as_table_mut().unwrap().set_position(0);

        if !doc["html"].is_table() {
            doc["html"] = doc["html"]
                .clone()
                .into_table()
                .unwrap_or_else(|_item| {
                    error!("Expected 'html' to be a table, but found: {}", doc["html"]);
                    panic!("Invalid configuration format for 'html'");
                })
                .into();
        }
        doc["html"].as_table_mut().unwrap().sort_values();

        std::fs::write(file_path, doc.to_string())?
    }

    Ok(())
}

/// Initializes the global configuration from the specified file path
///
/// # Arguments
/// * `config_path` - The path to the configuration file.
///
/// # Returns
/// Returns a `Result` indicating success or failure. If successful, a global `CONFIG` has been
/// initialized.
pub fn init_config(config_path: &str) -> Result<(), Error> {
    CONFIG.get_or_init(|| {
        Config::from_file(config_path).unwrap_or_else(|err| {
            error!("Failed to load config: {}", err);
            std::process::exit(1);
        })
    });
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Toml(toml_edit::TomlError),
    TomlSerialization(toml_edit::ser::Error),
    TomlDeserialization(toml_edit::de::Error),
}

// Display
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Toml(e) => write!(f, "TOML error: {}", e),
            Error::TomlSerialization(e) => write!(f, "TOML serialization error: {}", e),
            Error::TomlDeserialization(e) => write!(f, "TOML deserialization error: {}", e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<toml_edit::TomlError> for Error {
    fn from(err: toml_edit::TomlError) -> Self {
        Error::Toml(err)
    }
}

impl From<toml_edit::ser::Error> for Error {
    fn from(err: toml_edit::ser::Error) -> Self {
        Error::TomlSerialization(err)
    }
}

impl From<toml_edit::de::Error> for Error {
    fn from(err: toml_edit::de::Error) -> Self {
        Error::TomlDeserialization(err)
    }
}
