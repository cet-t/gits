const CONFIG_NAME: &str = "config.toml";
const CONFIG_DIR: &str = "gits";

#[derive(Debug, Clone, ::serde::Deserialize)]
pub struct Keymap {
    pub up: Vec<String>,
    pub down: Vec<String>,
    pub confirm: Vec<String>,
    pub cancel: Vec<String>,
}

#[derive(Debug, ::serde::Deserialize)]
pub struct Config {
    pub keymap: Option<Keymap>,
}

impl Config {
    pub fn load() -> ::anyhow::Result<Self> {
        let path = config_path()?;
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

fn config_path() -> ::anyhow::Result<std::path::PathBuf> {
    let mut path =
        dirs::config_dir().ok_or_else(|| anyhow::anyhow!("could not locate config directory"))?;
    path.push(CONFIG_DIR);
    path.push(CONFIG_NAME);
    Ok(path)
}
