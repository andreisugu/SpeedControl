use crate::STATE;
use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::{EventData, EventHandler, PlayerJoinEvent};

pub struct PlayerJoinHandler;

impl EventHandler<PlayerJoinEvent> for PlayerJoinHandler {
    fn handle(
        &self,
        _server: Server,
        data: EventData<PlayerJoinEvent>,
    ) -> EventData<PlayerJoinEvent> {
        let player = &data.player;
        let id = player.get_id();
        let uuid_str = format!("{:016x}{:016x}", id.high, id.low);

        let state = STATE.lock().unwrap();
        if let Some(player_data) = state.player_speeds.get(&uuid_str) {
            let mut applied_fly = None;
            let mut applied_walk = None;

            if let Some(fly_mult) = player_data.fly_multiplier {
                let mut abilities = player.get_abilities();
                abilities.fly_speed = 0.05 * fly_mult;
                player.set_abilities(abilities);
                applied_fly = Some(fly_mult);
            }

            if let Some(walk_mult) = player_data.walk_multiplier {
                let mut abilities = player.get_abilities();
                abilities.walk_speed = 0.1 * walk_mult;
                player.set_abilities(abilities);
                applied_walk = Some(walk_mult);
            }

            if applied_fly.is_some() || applied_walk.is_some() {
                tracing::info!(
                    player_name = player.get_name(),
                    player_uuid = %uuid_str,
                    fly_multiplier = ?applied_fly,
                    walk_multiplier = ?applied_walk,
                    "Reapplied player speed preferences on server join"
                );
            }
        }

        data
    }
}
