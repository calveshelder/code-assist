use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub llm: LlmConfig,
    pub editor: EditorConfig,
    pub git: GitConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditorConfig {
    pub default_editor: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitConfig {
    pub enable_git_features: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                api_url: "http://localhost:11434/api".to_string(),
                api_key: "".to_string(),
                model: "codellama".to_string(),
                temperature: 0.7,
                max_tokens: 2048,
            },
            editor: EditorConfig {
                default_editor: "vim".to_string(),
            },
            git: GitConfig {
                enable_git_features: true,
            },
        }
    }
}

pub fn load_or_create_config(config_path: &Path) -> Result<Config> {
    if !config_path.exists() {
        let config_dir = config_path.parent().unwrap();
        fs::create_dir_all(config_dir)?;
        
        let config = Config::default();
        let toml_string = toml::to_string_pretty(&config)?;
        
        let mut file = File::create(config_path)?;
        file.write_all(toml_string.as_bytes())?;
        
        return Ok(config);
    }
    
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let config: Config = toml::from_str(&contents)
        .context("Failed to parse config file")?;
    
    Ok(config)
}

pub fn update_config(
    config_path: &Path,
    api_url: &Option<String>,
    api_key: &Option<String>,
    model: &Option<String>,
) -> Result<()> {
    let mut config = load_or_create_config(config_path)?;
    
    if let Some(url) = api_url {
        config.llm.api_url = url.clone();
    }
    
    if let Some(key) = api_key {
        config.llm.api_key = key.clone();
    }
    
    if let Some(model_name) = model {
        config.llm.model = model_name.clone();
    }
    
    let toml_string = toml::to_string_pretty(&config)?;
    let mut file = File::create(config_path)?;
    file.write_all(toml_string.as_bytes())?;
    
    Ok(())
}
