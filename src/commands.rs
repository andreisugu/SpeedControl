use pumpkin_plugin_api::{
    Server,
    command_wit::{Arg, CommandError, CommandSender, ConsumedArgs, Number},
    commands::CommandHandler,
    text::{NamedColor, TextComponent},
};

use crate::STATE;
use crate::speed_type::SpeedType;

pub struct SpeedExecutor {
    pub speed_type: SpeedType,
    pub has_target: bool,
    pub is_reset: bool,
}

impl CommandHandler for SpeedExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        // 0. Check permission
        if !sender.has_permission(&server, self.speed_type.permission_node()) {
            return Err(CommandError::PermissionDenied);
        }

        // 1. Validate speed support upfront
        match self.speed_type {
            SpeedType::Fly | SpeedType::Walk => {}
            other_speed => {
                let limit_msg = TextComponent::text(&format!(
                    "Pumpkin's plugin API does not support {} speed modifiers yet.",
                    other_speed.name()
                ));
                return Err(CommandError::CommandFailed(limit_msg));
            }
        }

        // 2. Determine the multiplier value
        let multiplier: f32 = if self.is_reset {
            1.0
        } else {
            let arg = args.get_value("multiplier");
            match arg {
                Arg::Num(Ok(Number::Float32(val))) => val,
                Arg::Num(Ok(Number::Float64(val))) => val as f32,
                _ => {
                    let err_msg =
                        TextComponent::text("Invalid multiplier number! Use decimals like 1.5");
                    return Err(CommandError::CommandFailed(err_msg));
                }
            }
        };

        #[cfg(feature = "development-logs")]
        tracing::debug!(
            "SpeedExecutor execution context: speed_type={:?}, multiplier={}, has_target={}, is_reset={}",
            self.speed_type,
            multiplier,
            self.has_target,
            self.is_reset
        );

        // 3. Enforce safe boundaries (from config limits)
        {
            let state = STATE.lock().unwrap();
            match self.speed_type {
                SpeedType::Fly if multiplier > state.config.max_fly_speed_multiplier => {
                    tracing::warn!(
                        multiplier,
                        limit = state.config.max_fly_speed_multiplier,
                        "Multiplier exceeds config flying speed limit"
                    );
                    let err = TextComponent::text(&format!(
                        "Multiplier exceeds the server cap of {}x for flight speed.",
                        state.config.max_fly_speed_multiplier
                    ));
                    return Err(CommandError::CommandFailed(err));
                }
                SpeedType::Walk if multiplier > state.config.max_walk_speed_multiplier => {
                    tracing::warn!(
                        multiplier,
                        limit = state.config.max_walk_speed_multiplier,
                        "Multiplier exceeds config walking speed limit"
                    );
                    let err = TextComponent::text(&format!(
                        "Multiplier exceeds the server cap of {}x for walking speed.",
                        state.config.max_walk_speed_multiplier
                    ));
                    return Err(CommandError::CommandFailed(err));
                }
                _ => {}
            }
        }

        // 4. Resolve target player(s)
        let targets = if self.has_target {
            let target_arg = args.get_value("target");
            match target_arg {
                Arg::Players(players) => players,
                _ => {
                    let err_msg = TextComponent::text("Invalid target argument!");
                    return Err(CommandError::CommandFailed(err_msg));
                }
            }
        } else {
            if let Some(player) = sender.as_player() {
                vec![player]
            } else {
                let msg = TextComponent::text(
                    "This command must be executed by a player, or target player(s) must be specified.",
                );
                return Err(CommandError::CommandFailed(msg));
            }
        };

        if targets.is_empty() {
            let err_msg = TextComponent::text(
                "No target players found (players are offline or selection is empty).",
            );
            return Err(CommandError::CommandFailed(err_msg));
        }

        // Lock state to save changes to persistence db
        let mut state = STATE.lock().unwrap();

        // 5. Apply the modifications to the targets
        for player in &targets {
            let id = player.get_id();
            let uuid_str = format!("{:016x}{:016x}", id.high, id.low);
            let entry = state.player_speeds.entry(uuid_str.clone()).or_default();

            match self.speed_type {
                SpeedType::Fly => {
                    let mut abilities = player.get_abilities();
                    abilities.fly_speed = 0.05 * multiplier;
                    player.set_abilities(abilities);

                    // Update database
                    if self.is_reset {
                        entry.fly_multiplier = None;
                    } else {
                        entry.fly_multiplier = Some(multiplier);
                    }

                    tracing::info!(
                        player_name = player.get_name(),
                        player_uuid = %uuid_str,
                        multiplier,
                        "Set player flying speed"
                    );

                    // Notify sender
                    let sender_msg = TextComponent::text(&format!(
                        "Flight speed multiplier for {} set to: {} (raw speed: {})",
                        player.get_name(),
                        multiplier,
                        abilities.fly_speed
                    ));
                    sender_msg.color_named(NamedColor::Green);
                    let _ = sender.send_message(sender_msg);

                    // Notify target
                    if sender.get_name() != player.get_name() {
                        let target_msg = TextComponent::text(&format!(
                            "Your flight speed multiplier has been set to: {}",
                            multiplier
                        ));
                        target_msg.color_named(NamedColor::Green);
                        player.send_system_message(target_msg, false);
                    }
                }
                SpeedType::Walk => {
                    let mut abilities = player.get_abilities();
                    abilities.walk_speed = 0.1 * multiplier;
                    player.set_abilities(abilities);

                    // Update database
                    if self.is_reset {
                        entry.walk_multiplier = None;
                    } else {
                        entry.walk_multiplier = Some(multiplier);
                    }

                    tracing::info!(
                        player_name = player.get_name(),
                        player_uuid = %uuid_str,
                        multiplier,
                        "Set player walking speed"
                    );

                    // Notify sender
                    let sender_msg = TextComponent::text(&format!(
                        "Walking speed multiplier for {} set to: {} (raw speed: {})",
                        player.get_name(),
                        multiplier,
                        abilities.walk_speed
                    ));
                    sender_msg.color_named(NamedColor::Green);
                    let _ = sender.send_message(sender_msg);

                    // Notify target
                    if sender.get_name() != player.get_name() {
                        let target_msg = TextComponent::text(&format!(
                            "Your walking speed multiplier has been set to: {}",
                            multiplier
                        ));
                        target_msg.color_named(NamedColor::Green);
                        player.send_system_message(target_msg, false);
                    }
                }
                _ => unreachable!(),
            }
        }

        // Save DB file
        if let Err(err) = state.save_speeds() {
            tracing::error!(
                error = %err,
                "Failed to save speed adjustments to disk"
            );
            return Err(CommandError::CommandFailed(TextComponent::text(
                "Internal error: speed saved successfully in memory but failed to write to disk.",
            )));
        }

        Ok(targets.len() as i32)
    }
}

pub struct InfoExecutor {
    pub has_target: bool,
}

impl CommandHandler for InfoExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let targets = if self.has_target {
            let target_arg = args.get_value("target");
            match target_arg {
                Arg::Players(players) => players,
                _ => {
                    return Err(CommandError::CommandFailed(TextComponent::text(
                        "Invalid target argument!",
                    )));
                }
            }
        } else {
            if let Some(player) = sender.as_player() {
                vec![player]
            } else {
                return Err(CommandError::CommandFailed(TextComponent::text(
                    "This command must be executed by a player, or target player(s) must be specified.",
                )));
            }
        };

        if targets.is_empty() {
            return Err(CommandError::CommandFailed(TextComponent::text(
                "No target players found (players are offline or selection is empty).",
            )));
        }

        for player in &targets {
            let abilities = player.get_abilities();
            let fly_mult = abilities.fly_speed / 0.05;
            let walk_mult = abilities.walk_speed / 0.1;

            let header =
                TextComponent::text(&format!("--- Speed status for {} ---", player.get_name()));
            header.color_named(NamedColor::Gold);
            let _ = sender.send_message(header);

            let walk_msg = TextComponent::text(&format!(
                "  Walking Speed: {:.2}x (raw: {:.4})",
                walk_mult, abilities.walk_speed
            ));
            walk_msg.color_named(NamedColor::Yellow);
            let _ = sender.send_message(walk_msg);

            let fly_msg = TextComponent::text(&format!(
                "  Flying Speed: {:.2}x (raw: {:.4})",
                fly_mult, abilities.fly_speed
            ));
            fly_msg.color_named(NamedColor::Yellow);
            let _ = sender.send_message(fly_msg);
        }

        Ok(targets.len() as i32)
    }
}

pub struct ClearExecutor {
    pub has_target: bool,
}

impl CommandHandler for ClearExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let targets = if self.has_target {
            let target_arg = args.get_value("target");
            match target_arg {
                Arg::Players(players) => players,
                _ => {
                    return Err(CommandError::CommandFailed(TextComponent::text(
                        "Invalid target argument!",
                    )));
                }
            }
        } else {
            if let Some(player) = sender.as_player() {
                vec![player]
            } else {
                return Err(CommandError::CommandFailed(TextComponent::text(
                    "This command must be executed by a player, or target player(s) must be specified.",
                )));
            }
        };

        if targets.is_empty() {
            return Err(CommandError::CommandFailed(TextComponent::text(
                "No target players found (players are offline or selection is empty).",
            )));
        }

        let mut state = STATE.lock().unwrap();

        for player in &targets {
            let id = player.get_id();
            let uuid_str = format!("{:016x}{:016x}", id.high, id.low);

            // Clear persistence
            state.player_speeds.remove(&uuid_str);

            // Reset abilities
            let mut abilities = player.get_abilities();
            abilities.fly_speed = 0.05;
            abilities.walk_speed = 0.1;
            player.set_abilities(abilities);

            tracing::info!(
                player_name = player.get_name(),
                player_uuid = %uuid_str,
                "Cleared player speed modifiers"
            );

            // Notify
            let sender_msg = TextComponent::text(&format!(
                "Cleared all speed modifiers for {}",
                player.get_name()
            ));
            sender_msg.color_named(NamedColor::Green);
            let _ = sender.send_message(sender_msg);

            if sender.get_name() != player.get_name() {
                let target_msg = TextComponent::text(
                    "Your speed modifiers have been cleared and reset to defaults.",
                );
                target_msg.color_named(NamedColor::Green);
                player.send_system_message(target_msg, false);
            }
        }

        if let Err(err) = state.save_speeds() {
            tracing::error!(
                error = %err,
                "Failed to write persistence file during clear speed operation"
            );
            return Err(CommandError::CommandFailed(TextComponent::text(
                "Internal error: cleared speeds in memory, but failed to write changes to disk.",
            )));
        }

        Ok(targets.len() as i32)
    }
}

pub struct ReloadExecutor;

impl CommandHandler for ReloadExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        if !sender.has_permission(&server, "SpeedControl:command.reload") {
            return Err(CommandError::PermissionDenied);
        }

        let mut state = STATE.lock().unwrap();
        state.reload_config();

        tracing::info!("Plugin configuration hot-reloaded by administrator");

        let msg = TextComponent::text("Plugin configuration reloaded successfully.");
        msg.color_named(NamedColor::Green);
        let _ = sender.send_message(msg);

        Ok(1)
    }
}

#[cfg(feature = "extras")]
pub struct ExperimentalExecutor;

#[cfg(feature = "extras")]
impl CommandHandler for ExperimentalExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let msg =
            TextComponent::text("Running in experimental mode! Speed multipliers are augmented.");
        msg.color_named(NamedColor::LightPurple);
        let _ = sender.send_message(msg);
        Ok(1)
    }
}
