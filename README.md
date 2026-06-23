# ⚡ SpeedControl for Pumpkin Minecraft Server

[![CI](https://github.com/andreisugu/SpeedControl/actions/workflows/ci.yml/badge.svg)](https://github.com/andreisugu/SpeedControl/actions/workflows/ci.yml)
[![Language](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Target](https://img.shields.io/badge/Target-WASM%20(WASI%20Preview2)-blue.svg)](https://wasi.dev/)
[![Security](https://img.shields.io/badge/Security-cargo--audit%20%2F%20deny-green.svg)](https://github.com/rustsec/rustsec)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-lightgrey.svg)](#license)

`SpeedControl` is a high-performance, enterprise-grade WebAssembly (WASM) plugin written in Rust for the [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) Minecraft server. It offers server administrators fine-grained control over player movement speeds with persistent databases, configurable safety boundaries, auto-generated JSON schemas, structured telemetry, and zero-cost feature flags.

---

## ✨ Features

- **🌐 Unified `/speed` Command:** Centralizes flight, walk, and experimental speed control under one cohesive command branch.
- **💾 Persistent Speed DB:** Saves custom speeds to a persistent database (`saved_speeds.json`) and seamlessly reapplies them when players join.
- **🛡️ Fail-Fast Configuration Validation:** Automatically generates a JSON Schema (`schema.json`) in the data directory on startup. Validates the `config.json` boundaries on load and safely falls back to defaults instead of panicking.
- **🔒 Enterprise Security & Auditing:** Audits dependencies via `cargo-deny` and `cargo-audit` to prevent license leaks (GPL copyleft block) and insecure crates.
- **⚡ Zero-Cost Cargo Feature Flags:** Selectively compile debug logs and experimental subcommands using Rust features, keeping production binaries lightweight.
- **🤖 Modern CI/CD Workflow:** Automatically compiles, formats, and executes unit tests natively across Linux & Windows on every pull request, and publishes production artifacts to GitHub Releases on version tags (`v*`).

---

## 🛠️ Commands & Permissions

By default, administrative command roots require Minecraft **Op Level 2** (or matching permission nodes via permission management plugins).

| Command | Alias / Subcommand | Description | Permission Node |
| :--- | :--- | :--- | :--- |
| `/speed fly <multiplier> [player]` | `/flyspeed`, `/fs` | Sets flight speed multiplier (e.g., `1.5` for 150%) | `SpeedControl:command.flyspeed` |
| `/speed walk <multiplier> [player]` | `/walkspeed`, `/ws` | Sets walking speed multiplier (e.g., `2.0` for 200%) | `SpeedControl:command.walkspeed` |
| `/speed <type> reset [player]` | — | Resets target's flight/walk speed to vanilla | `SpeedControl:command.flyspeed`/`walkspeed` |
| `/speed info [player]` | — | Displays active multiplier values and raw speed attributes | `SpeedControl:command.speed` |
| `/speed clear [player]` | — | Resets target speed modifiers and deletes database entry | `SpeedControl:command.speed` |
| `/speed reload` | — | Hot-reloads configuration limits without restarting | `SpeedControl:command.reload` |
| `/speed experimental` | *(with `extras` feature)* | Executes experimental speed logic sub-routines | `SpeedControl:command.speed` |

> [!NOTE]
> Vanilla default multipliers are **1.0x** (raw walk speed: `0.1`, raw fly speed: `0.05`). Supported command inputs range up to `100.0x` within API boundaries, subject to config limitations.

---

## ⚙️ Configuration & Schema Validation

On startup, the plugin creates its data folder (`plugins/SpeedControl`) and generates:

1. **`config.json`**:
   ```json
   {
     "version": 1,
     "max_fly_speed_multiplier": 10.0,
     "max_walk_speed_multiplier": 10.0
   }
   ```
2. **`schema.json`**: An auto-generated JSON Schema derived from the Rust structure, offering autocomplete validation inside modern JSON editors (like VS Code).

If configuration parameters fail validation (e.g., negative caps or values exceeding physical limits), the loader logs the error and gracefully applies safe defaults to keep the server stable.

---

## 🧩 Cargo Feature Flags

You can customize compilation features to strip out code and maintain zero-cost binary footprints:

- **`development-logs`**: Enables detailed `tracing::debug` instrumentation during persistence I/O, player join events, and command executors.
- **`extras`**: Enables experimental features (like the `/speed experimental` subcommand executor).

Compile with features:
```bash
cargo build --release --features "extras,development-logs"
```

---

## 🏗️ Local Development & Automation

Local coordination scripts are provided for both Windows (PowerShell) and Linux/macOS (Bash) environments to manage linting, testing, and deployment:

### Windows (PowerShell)
```powershell
# 1. Standard build, copy to target server, and pre-approve WASM permissions
.\build.ps1

# 2. Build with custom feature flags
.\build.ps1 -Features "extras,development-logs"

# 3. Execute quality gates: cargo fmt, clippy, local deny/audit scans, and native tests
.\build.ps1 -Test -Lint

# 4. Clean build outputs, target folders, and duplicate build-checks
.\build.ps1 -Clean
```

### Linux & macOS (Bash)
```bash
# 1. Standard build, copy to target server, and pre-approve WASM permissions
./build.sh

# 2. Build with custom feature flags
./build.sh -f "extras,development-logs"

# 3. Execute quality gates: cargo fmt, clippy, local deny/audit scans, and native tests
./build.sh -l -t

# 4. Clean build outputs, target folders, and duplicate build-checks
./build.sh -c
```

---

## 🧪 Testing & Mocking

The business logic, database persistence, and configuration parsers are fully testable on standard native host architectures.
- **Mocking:** Uses `mockall` to generate high-fidelity mock implementations of the `SpeedStore` database trait. This allows test suites to execute without ever touching filesystems or calling Pumpkin's WASM host imports.
- Run tests natively on your development machine:
  ```bash
  cargo test --target <your-native-host-triple> --all-features
  ```

---

## 📄 License

This project is dual-licensed under either:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
