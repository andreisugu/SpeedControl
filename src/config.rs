use crate::errors::SpeedControlError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq)]
pub struct PluginConfig {
    pub version: u32,
    pub max_fly_speed_multiplier: f32,
    pub max_walk_speed_multiplier: f32,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            version: 1,
            max_fly_speed_multiplier: 10.0,
            max_walk_speed_multiplier: 10.0,
        }
    }
}

impl PluginConfig {
    pub fn validate(&self) -> Result<(), SpeedControlError> {
        if self.max_fly_speed_multiplier <= 0.0 || self.max_fly_speed_multiplier > 100.0 {
            return Err(SpeedControlError::ConfigValidation(format!(
                "max_fly_speed_multiplier must be between 0.1 and 100.0, got {}",
                self.max_fly_speed_multiplier
            )));
        }
        if self.max_walk_speed_multiplier <= 0.0 || self.max_walk_speed_multiplier > 100.0 {
            return Err(SpeedControlError::ConfigValidation(format!(
                "max_walk_speed_multiplier must be between 0.1 and 100.0, got {}",
                self.max_walk_speed_multiplier
            )));
        }
        Ok(())
    }

    pub fn load(data_folder: &str) -> Self {
        let config_path = format!("{}/config.json", data_folder);
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<PluginConfig>(&content) {
                    Ok(mut cfg) => {
                        // Migrate legacy formats
                        if cfg.version < 1 {
                            cfg.version = 1;
                            let migrated = cfg.clone();
                            if let Err(err) = migrated.save(data_folder) {
                                tracing::error!("Failed to save migrated config.json: {}", err);
                            }
                        }
                        // Validate config parameters
                        if let Err(err) = cfg.validate() {
                            tracing::error!(
                                "Invalid configuration: {}. Falling back to default settings.",
                                err
                            );
                            Self::default()
                        } else {
                            tracing::info!(
                                "SpeedControl configuration loaded (version {}).",
                                cfg.version
                            );
                            cfg
                        }
                    }
                    Err(err) => {
                        tracing::error!(
                            "Failed to parse config.json: {}. Using default values.",
                            err
                        );
                        Self::default()
                    }
                }
            }
            Err(err) => {
                let cfg = Self::default();
                if std::path::Path::new(&config_path).exists() {
                    tracing::error!("Failed to read config.json: {}", err);
                } else {
                    if let Err(save_err) = cfg.save(data_folder) {
                        tracing::error!("Failed to save default config.json: {}", save_err);
                    }
                }
                cfg
            }
        }
    }

    pub fn save(&self, data_folder: &str) -> Result<(), std::io::Error> {
        let config_path = format!("{}/config.json", data_folder);
        let content = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(config_path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_validation() {
        let cfg = PluginConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_config_invalid_validation() {
        let cfg = PluginConfig {
            max_fly_speed_multiplier: -1.0,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());

        let cfg = PluginConfig {
            max_fly_speed_multiplier: 101.0,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());

        let cfg = PluginConfig {
            max_fly_speed_multiplier: 10.0,
            max_walk_speed_multiplier: 0.0,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = std::env::temp_dir().join("speedcontrol_test_config");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let folder_str = temp_dir.to_string_lossy().to_string();

        let cfg = PluginConfig {
            version: 1,
            max_fly_speed_multiplier: 5.5,
            max_walk_speed_multiplier: 8.2,
        };

        cfg.save(&folder_str).unwrap();

        let loaded = PluginConfig::load(&folder_str);
        assert_eq!(cfg, loaded);

        std::fs::remove_dir_all(temp_dir).unwrap();
    }
}
