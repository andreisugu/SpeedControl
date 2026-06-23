use pumpkin_plugin_api::events::{EventHandler, EventData, PlayerJoinEvent};
use pumpkin_plugin_api::Server;
use crate::STATE;

pub struct PlayerJoinHandler;

impl EventHandler<PlayerJoinEvent> for PlayerJoinHandler {
    fn handle(&self, _server: Server, data: EventData<PlayerJoinEvent>) -> EventData<PlayerJoinEvent> {
        let player = &data.player;
        let id = player.get_id();
        let uuid_str = format!("{:016x}{:016x}", id.high, id.low);

        let state = STATE.lock().unwrap();
        if let Some(player_data) = state.player_speeds.get(&uuid_str) {
            if let Some(fly_mult) = player_data.fly_multiplier {
                let mut abilities = player.get_abilities();
                abilities.fly_speed = 0.05 * fly_mult;
                player.set_abilities(abilities);
            }
            if let Some(walk_mult) = player_data.walk_multiplier {
                let mut abilities = player.get_abilities();
                abilities.walk_speed = 0.1 * walk_mult;
                player.set_abilities(abilities);
            }
        }

        data
    }
}
