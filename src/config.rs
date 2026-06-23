use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub fn load(data_folder: &str) -> Self {
        let config_path = format!("{}/config.json", data_folder);
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<PluginConfig>(&content) {
                    Ok(mut cfg) => {
                        if cfg.version < 1 {
                            cfg.version = 1;
                            let migrated = cfg.clone();
                            if let Err(err) = migrated.save(data_folder) {
                                tracing::error!("Failed to save migrated config.json: {}", err);
                            }
                        }
                        tracing::info!("SpeedControl configuration loaded (version {}).", cfg.version);
                        cfg
                    }
                    Err(err) => {
                        tracing::error!("Failed to parse config.json: {}. Using default values.", err);
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
