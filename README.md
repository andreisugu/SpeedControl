# Pumpkin SpeedControl Plugin

A WebAssembly-compatible Rust plugin for the [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) Minecraft server. It implements commands to modify, persist, and limit player movement speeds (walking and flying).

---

## Features

- **Unified `/speed` command**: Access and control walking, flying, and legacy speed types from one centralized command.
- **Legacy Commands & Aliases**: Supports traditional commands `/flyspeed` (`/fs`) and `/walkspeed` (`/ws`) for console/player familiarity.
- **Persistent Multipliers**: Player speed settings are saved to a persistent database (`saved_speeds.json`) and automatically reapplied when players join.
- **Configurable Speed Caps**: Maximum allowed multipliers are defined inside `config.json` and validated on input.
- **Subcommands**:
  - `/speed info [player]`: View active multiplier values and raw speed attributes.
  - `/speed clear [player]`: Reset speeds to vanilla defaults and delete player persistence records.
  - `/speed reload`: Hot-reload settings on the fly without server restarts.
- **SOLID Clean Design**: Clean, modular structure with distinct config, persistence, command, event, and domain models. See [ARCHITECTURE.md](file:///c:/VSCodeProjects/PumpkinPlugins/SpeedControl/ARCHITECTURE.md).
- **Automated Deploy Tool**: Quick clean, compilation, deployment, and permission pre-approval scripts. See [WORKFLOW.md](file:///c:/VSCodeProjects/PumpkinPlugins/SpeedControl/WORKFLOW.md).

---

## Commands & Permissions

All administrative commands require **Op Level 2** by default, or specific permission nodes if a permission manager is present.

| Command | Description | Permission Node |
| :--- | :--- | :--- |
| `/speed fly <multiplier> [player]` | Set flight speed multiplier | `SpeedControl:command.flyspeed` |
| `/speed walk <multiplier> [player]` | Set walking speed multiplier | `SpeedControl:command.walkspeed` |
| `/speed <type> reset [player]` | Reset flight/walking speed to vanilla | `SpeedControl:command.flyspeed` / `walkspeed` |
| `/speed info [player]` | View target player's multipliers and raw attributes | `SpeedControl:command.speed` |
| `/speed clear [player]` | Reset target's speeds and delete database record | `SpeedControl:command.speed` |
| `/speed reload` | Reloads `config.json` limits into memory | `SpeedControl:command.reload` |
| `/flyspeed <multiplier> [player]` | Legacy alias for setting flight speed | `SpeedControl:command.flyspeed` |
| `/walkspeed <multiplier> [player]` | Legacy alias for setting walking speed | `SpeedControl:command.walkspeed` |

*Vanilla default multipliers are `1.0x` (raw walking speed: `0.1`, raw flying speed: `0.05`).*

---

## Configuration (`config.json`)

The config file is automatically generated inside the plugin's data directory (`plugins/SpeedControl/config.json`) on first load:

```json
{
  "version": 1,
  "max_fly_speed_multiplier": 10.0,
  "max_walk_speed_multiplier": 10.0
}
```

- `version`: Configuration schema version. Used for auto-migration.
- `max_fly_speed_multiplier`: The maximum multiplier allowed for flight speeds (aborts command if exceeded).
- `max_walk_speed_multiplier`: The maximum multiplier allowed for walking speeds (aborts command if exceeded).

---

## Quick Compilation & Deployment

A unified script `build.ps1` is provided in the project root to clean intermediate targets, compile, deploy, and pre-approve WASM sandboxing permissions:

```powershell
# 1. Standard build, copy to server, and pre-approve permissions
.\build.ps1

# 2. Clean build (removes duplicate target folders and object caches)
.\build.ps1 -Clean

# 3. Dev / Debug profile build (faster compilation)
.\build.ps1 -Dev
```

For manual build setups, path-resolution gotchas, and troubleshooting, see [WORKFLOW.md](file:///c:/VSCodeProjects/PumpkinPlugins/SpeedControl/WORKFLOW.md).
