use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConfigValue {
    Bool(bool),
    Float(f64),
    Int(i64),
    String(String),
}

impl ConfigValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(v) => Some(*v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigParam {
    pub key: &'static str,
    pub label: &'static str,
    pub default: ConfigValue,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub analyzers: HashMap<String, HashMap<String, ConfigValue>>,
    #[serde(default)]
    pub outputs: HashMap<String, HashMap<String, ConfigValue>>,
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("pdf_analyzer").join("config.toml"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| fs::read_to_string(&path).ok())
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path().ok_or_else(|| {
            AppError::ConfigError("Could not determine config directory".to_string())
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::ConfigError(e.to_string()))?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn get_analyzer_value(&self, analyzer_id: &str, key: &str) -> Option<&ConfigValue> {
        self.analyzers.get(analyzer_id)?.get(key)
    }

    pub fn set_analyzer_value(&mut self, analyzer_id: &str, key: &str, value: ConfigValue) {
        self.analyzers
            .entry(analyzer_id.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }

    pub fn get_output_value(&self, output_id: &str, key: &str) -> Option<&ConfigValue> {
        self.outputs.get(output_id)?.get(key)
    }

    pub fn set_output_value(&mut self, output_id: &str, key: &str, value: ConfigValue) {
        self.outputs
            .entry(output_id.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }
}
