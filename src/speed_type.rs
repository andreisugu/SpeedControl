#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedType {
    Fly,
    Walk,
    Elytra,
    Swim,
    Attack,
    Sneak,
    Mount,
    Mining,
}

impl SpeedType {
    pub fn name(self) -> &'static str {
        match self {
            SpeedType::Fly => "flying",
            SpeedType::Walk => "walking",
            SpeedType::Elytra => "elytra glide",
            SpeedType::Swim => "swimming",
            SpeedType::Attack => "attack cooldown",
            SpeedType::Sneak => "sneaking",
            SpeedType::Mount => "mount",
            SpeedType::Mining => "mining",
        }
    }

    pub fn permission_node(self) -> &'static str {
        match self {
            SpeedType::Fly => "SpeedControl:command.flyspeed",
            SpeedType::Walk => "SpeedControl:command.walkspeed",
            SpeedType::Elytra => "SpeedControl:command.elytraspeed",
            SpeedType::Swim => "SpeedControl:command.swimspeed",
            SpeedType::Attack => "SpeedControl:command.attackspeed",
            SpeedType::Sneak => "SpeedControl:command.sneakspeed",
            SpeedType::Mount => "SpeedControl:command.mountspeed",
            SpeedType::Mining => "SpeedControl:command.miningspeed",
        }
    }
}
