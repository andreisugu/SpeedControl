use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct PlayerSpeedData {
    pub fly_multiplier: Option<f32>,
    pub walk_multiplier: Option<f32>,
}

pub trait SpeedStore: Send + Sync {
    fn load_speeds(&self) -> HashMap<String, PlayerSpeedData>;
    fn save_speeds(&self, speeds: &HashMap<String, PlayerSpeedData>);
}

pub struct JsonPlayerSpeedStore {
    data_folder: String,
}

impl JsonPlayerSpeedStore {
    pub fn new(data_folder: String) -> Self {
        Self { data_folder }
    }

    fn file_path(&self) -> String {
        format!("{}/saved_speeds.json", self.data_folder)
    }
}

impl SpeedStore for JsonPlayerSpeedStore {
    fn load_speeds(&self) -> HashMap<String, PlayerSpeedData> {
        if self.data_folder.is_empty() {
            return HashMap::new();
        }
        let path = self.file_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<HashMap<String, PlayerSpeedData>>(&content) {
                    Ok(speeds) => speeds,
                    Err(err) => {
                        tracing::error!("Failed to parse saved_speeds.json: {}", err);
                        HashMap::new()
                    }
                }
            }
            Err(err) => {
                if std::path::Path::new(&path).exists() {
                    tracing::error!("Failed to read saved_speeds.json: {}", err);
                }
                HashMap::new()
            }
        }
    }

    fn save_speeds(&self, speeds: &HashMap<String, PlayerSpeedData>) {
        if self.data_folder.is_empty() {
            return;
        }
        let path = self.file_path();
        match serde_json::to_string_pretty(speeds) {
            Ok(content) => {
                if let Err(err) = std::fs::write(&path, content) {
                    tracing::error!("Failed to write saved_speeds.json: {}", err);
                }
            }
            Err(err) => {
                tracing::error!("Failed to serialize player speeds: {}", err);
            }
        }
    }
}
