use crate::errors::SpeedControlError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct PlayerSpeedData {
    pub fly_multiplier: Option<f32>,
    pub walk_multiplier: Option<f32>,
}

#[cfg_attr(test, mockall::automock)]
pub trait SpeedStore: Send + Sync {
    fn load_speeds(&self) -> Result<HashMap<String, PlayerSpeedData>, SpeedControlError>;
    fn save_speeds(
        &self,
        speeds: &HashMap<String, PlayerSpeedData>,
    ) -> Result<(), SpeedControlError>;
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
    fn load_speeds(&self) -> Result<HashMap<String, PlayerSpeedData>, SpeedControlError> {
        if self.data_folder.is_empty() {
            return Ok(HashMap::new());
        }
        let path = self.file_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let speeds = serde_json::from_str::<HashMap<String, PlayerSpeedData>>(&content)?;
                Ok(speeds)
            }
            Err(err) => {
                if std::path::Path::new(&path).exists() {
                    Err(SpeedControlError::Io(err))
                } else {
                    Ok(HashMap::new())
                }
            }
        }
    }

    fn save_speeds(
        &self,
        speeds: &HashMap<String, PlayerSpeedData>,
    ) -> Result<(), SpeedControlError> {
        if self.data_folder.is_empty() {
            return Ok(());
        }
        let path = self.file_path();
        let content = serde_json::to_string_pretty(speeds)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_store_load_save() {
        let temp_dir = std::env::temp_dir().join("speedcontrol_test_persistence");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let folder_str = temp_dir.to_string_lossy().to_string();

        let store = JsonPlayerSpeedStore::new(folder_str);

        // Initially empty
        let initial = store.load_speeds().unwrap();
        assert!(initial.is_empty());

        // Save some speeds
        let mut speeds = HashMap::new();
        speeds.insert(
            "player-uuid-1".to_string(),
            PlayerSpeedData {
                fly_multiplier: Some(2.5),
                walk_multiplier: Some(1.2),
            },
        );

        store.save_speeds(&speeds).unwrap();

        // Load back and assert
        let loaded = store.load_speeds().unwrap();
        assert_eq!(
            loaded.get("player-uuid-1").unwrap().fly_multiplier,
            Some(2.5)
        );
        assert_eq!(
            loaded.get("player-uuid-1").unwrap().walk_multiplier,
            Some(1.2)
        );

        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_mock_store() {
        // Example of using mockall to mock the SpeedStore trait
        let mut mock = MockSpeedStore::new();

        mock.expect_load_speeds().times(1).returning(|| {
            let mut map = HashMap::new();
            map.insert(
                "mocked-uuid".to_string(),
                PlayerSpeedData {
                    fly_multiplier: Some(4.0),
                    walk_multiplier: None,
                },
            );
            Ok(map)
        });

        let loaded = mock.load_speeds().unwrap();
        assert_eq!(loaded.get("mocked-uuid").unwrap().fly_multiplier, Some(4.0));
    }
}
