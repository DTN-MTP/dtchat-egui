use serde::de::DeserializeOwned;
use std::fs;

pub fn load_yaml_from_file<T: DeserializeOwned>(
    file_path: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(file_path)?;
    let config: T = serde_yaml::from_str(&config_str)?;
    Ok(config)
}
