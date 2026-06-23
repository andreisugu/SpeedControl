use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};

pub mod config;
pub mod persistence;
pub mod speed_type;
pub mod commands;
pub mod events;

use config::PluginConfig;
use persistence::{PlayerSpeedData, SpeedStore, JsonPlayerSpeedStore};
use pumpkin_plugin_api::{
    command_wit::{ArgumentType, Command, CommandNode},
    Context, Plugin, PluginMetadata,
};
use pumpkin_plugin_api::permission::{Permission, PermissionDefault, PermissionLevel};
use pumpkin_plugin_api::events::EventPriority;

pub struct PluginState {
    pub data_folder: String,
    pub config: PluginConfig,
    pub player_speeds: HashMap<String, PlayerSpeedData>,
    pub store: Option<JsonPlayerSpeedStore>,
}

impl PluginState {
    fn new() -> Self {
        Self {
            data_folder: String::new(),
            config: PluginConfig::default(),
            player_speeds: HashMap::new(),
            store: None,
        }
    }

    fn init(&mut self, data_folder: String) {
        self.data_folder = data_folder;
        if let Err(err) = std::fs::create_dir_all(&self.data_folder) {
            tracing::error!("Failed to create data folder '{}': {}", self.data_folder, err);
        }
        
        // Initialize config
        self.config = PluginConfig::load(&self.data_folder);
        
        // Initialize store and load speeds
        let store = JsonPlayerSpeedStore::new(self.data_folder.clone());
        self.player_speeds = store.load_speeds();
        self.store = Some(store);
    }

    pub fn reload_config(&mut self) {
        self.config = PluginConfig::load(&self.data_folder);
    }

    pub fn save_speeds(&self) {
        if let Some(ref store) = self.store {
            store.save_speeds(&self.player_speeds);
        }
    }
}

pub static STATE: LazyLock<Mutex<PluginState>> = LazyLock::new(|| Mutex::new(PluginState::new()));

struct SpeedPlugin;

impl Plugin for SpeedPlugin {
    fn new() -> Self {
        SpeedPlugin
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "SpeedControl".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["Developer".into()],
            description: "Controls walking, flying, and other speeds with persistence, caps, and diagnostics.".into(),
            dependencies: Vec::new(),
            permissions: vec![
                "fs.read.data".into(),
                "fs.write.data".into(),
            ],
        }
    }

    fn on_load(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        let folder = context.get_data_folder();
        tracing::info!("SpeedControl Plugin loading... Data folder: '{}'", folder);

        // Initialize state with data folder
        {
            let mut state = STATE.lock().unwrap();
            state.init(folder);
        }

        // Register player join handler
        context.register_event_handler(
            events::PlayerJoinHandler,
            EventPriority::Normal,
            false,
        )?;

        // 1. Register Permissions
        let permissions = vec![
            ("SpeedControl:command.speed", "Allows modifying game speeds"),
            ("SpeedControl:command.flyspeed", "Allows setting fly speed"),
            ("SpeedControl:command.walkspeed", "Allows setting walk speed"),
            ("SpeedControl:command.elytraspeed", "Allows setting elytra speed"),
            ("SpeedControl:command.swimspeed", "Allows setting swim speed"),
            ("SpeedControl:command.attackspeed", "Allows setting combat attack speed"),
            ("SpeedControl:command.sneakspeed", "Allows setting sneak speed"),
            ("SpeedControl:command.mountspeed", "Allows setting mount speed"),
            ("SpeedControl:command.miningspeed", "Allows setting mining speed"),
            ("SpeedControl:command.reload", "Allows reloading configuration values"),
        ];

        for (node, desc) in permissions {
            context.register_permission(&Permission {
                node: node.to_string(),
                description: desc.to_string(),
                default: PermissionDefault::Op(PermissionLevel::Two),
                children: Vec::new(),
            })?;
        }

        // 2. Build and Register Command Trees
        use speed_type::SpeedType;
        use commands::{SpeedExecutor, InfoExecutor, ClearExecutor, ReloadExecutor};

        // Unified /speed Command
        let speed_cmd = Command::new(
            &["speed".to_string(), "spd".to_string()],
            "Modify various speed multipliers"
        );

        let types = vec![
            ("fly", SpeedType::Fly),
            ("walk", SpeedType::Walk),
            ("elytra", SpeedType::Elytra),
            ("swim", SpeedType::Swim),
            ("attack", SpeedType::Attack),
            ("sneak", SpeedType::Sneak),
            ("mount", SpeedType::Mount),
            ("mining", SpeedType::Mining),
        ];

        for (literal_name, speed_type) in types {
            let type_node = CommandNode::literal(literal_name);

            // /speed <type> reset
            let reset_node = CommandNode::literal("reset")
                .execute(SpeedExecutor {
                    speed_type,
                    has_target: false,
                    is_reset: true,
                });
            
            // /speed <type> reset <target>
            let reset_target_node = CommandNode::argument("target", &ArgumentType::Players)
                .execute(SpeedExecutor {
                    speed_type,
                    has_target: true,
                    is_reset: true,
                });
            reset_node.then(reset_target_node);

            // /speed <type> <multiplier>
            let mult_node = CommandNode::argument("multiplier", &ArgumentType::Float((Some(0.0), Some(100.0))))
                .execute(SpeedExecutor {
                    speed_type,
                    has_target: false,
                    is_reset: false,
                });

            // /speed <type> <multiplier> <target>
            let mult_target_node = CommandNode::argument("target", &ArgumentType::Players)
                .execute(SpeedExecutor {
                    speed_type,
                    has_target: true,
                    is_reset: false,
                });
            mult_node.then(mult_target_node);

            type_node.then(reset_node);
            type_node.then(mult_node);
            speed_cmd.then(type_node);
        }

        // /speed info
        let info_node = CommandNode::literal("info")
            .execute(InfoExecutor { has_target: false });
        // /speed info <target>
        let info_target_node = CommandNode::argument("target", &ArgumentType::Players)
            .execute(InfoExecutor { has_target: true });
        info_node.then(info_target_node);
        speed_cmd.then(info_node);

        // /speed clear
        let clear_node = CommandNode::literal("clear")
            .execute(ClearExecutor { has_target: false });
        // /speed clear <target>
        let clear_target_node = CommandNode::argument("target", &ArgumentType::Players)
            .execute(ClearExecutor { has_target: true });
        clear_node.then(clear_target_node);
        speed_cmd.then(clear_node);

        // /speed reload
        let reload_node = CommandNode::literal("reload")
            .execute(ReloadExecutor);
        speed_cmd.then(reload_node);

        context.register_command(speed_cmd, "SpeedControl:command.speed");

        // Fly Speed Command (Legacy / Alias support)
        let fly_legacy = Command::new(
            &["flyspeed".to_string(), "fs".to_string()],
            "Set flight speed multiplier"
        );
        let fly_reset = CommandNode::literal("reset")
            .execute(SpeedExecutor {
                speed_type: SpeedType::Fly,
                has_target: false,
                is_reset: true,
            });
        let fly_reset_target = CommandNode::argument("target", &ArgumentType::Players)
            .execute(SpeedExecutor {
                speed_type: SpeedType::Fly,
                has_target: true,
                is_reset: true,
            });
        fly_reset.then(fly_reset_target);

        let fly_mult = CommandNode::argument("multiplier", &ArgumentType::Float((Some(0.0), Some(100.0))))
            .execute(SpeedExecutor {
                speed_type: SpeedType::Fly,
                has_target: false,
                is_reset: false,
            });
        let fly_mult_target = CommandNode::argument("target", &ArgumentType::Players)
            .execute(SpeedExecutor {
                speed_type: SpeedType::Fly,
                has_target: true,
                is_reset: false,
            });
        fly_mult.then(fly_mult_target);
        fly_legacy.then(fly_reset);
        fly_legacy.then(fly_mult);
        context.register_command(fly_legacy, "SpeedControl:command.flyspeed");

        // Walk Speed Command (Legacy / Alias support)
        let walk_legacy = Command::new(
            &["walkspeed".to_string(), "ws".to_string()],
            "Set walking speed multiplier"
        );
        let walk_reset = CommandNode::literal("reset")
            .execute(SpeedExecutor {
                speed_type: SpeedType::Walk,
                has_target: false,
                is_reset: true,
            });
        let walk_reset_target = CommandNode::argument("target", &ArgumentType::Players)
            .execute(SpeedExecutor {
                speed_type: SpeedType::Walk,
                has_target: true,
                is_reset: true,
            });
        walk_reset.then(walk_reset_target);

        let walk_mult = CommandNode::argument("multiplier", &ArgumentType::Float((Some(0.0), Some(100.0))))
            .execute(SpeedExecutor {
                speed_type: SpeedType::Walk,
                has_target: false,
                is_reset: false,
            });
        let walk_mult_target = CommandNode::argument("target", &ArgumentType::Players)
            .execute(SpeedExecutor {
                speed_type: SpeedType::Walk,
                has_target: true,
                is_reset: false,
            });
        walk_mult.then(walk_mult_target);
        walk_legacy.then(walk_reset);
        walk_legacy.then(walk_mult);
        context.register_command(walk_legacy, "SpeedControl:command.walkspeed");

        Ok(())
    }

    fn on_unload(&mut self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        tracing::info!("SpeedControl Plugin unloaded.");
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(SpeedPlugin);
