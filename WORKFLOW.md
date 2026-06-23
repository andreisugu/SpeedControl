# SpeedControl Plugin Workflow & Debugging Guide

This guide documents the development, compilation, and troubleshooting workflow for the `SpeedControl` WebAssembly plugin, designed to prevent future configuration and WASI permission headaches.

---

## 1. Compilation & Deployment

A unified build and deployment script `build.ps1` is provided in the project root to automate cleaning, compiling, deploying, and pre-approving permissions.

### Prerequisites
Ensure the `wasm32-wasip2` target is installed:
```bash
rustup target add wasm32-wasip2
```

### Standard Build & Deploy (Release)
To compile a release build, deploy it to the Minecraft server's plugins directory, and pre-approve permissions:
```powershell
.\build.ps1
```

### Clean & Deploy (Release)
If you want to clear old build caches and remove redundant/messy target folders (like `target_build`, `build_check`, etc.):
```powershell
.\build.ps1 -Clean
```

### Dev / Debug Build & Deploy
To compile a debug build for faster compilation during development:
```powershell
.\build.ps1 -Dev
```

### Custom Deploy Directory
To deploy to a server in a different path:
```powershell
.\build.ps1 -DeployDir "D:\MinecraftServers\PumpkinServer\plugins"
```

---

## 2. WASI Directory Permissions (Crucial)

WebAssembly plugins run in a sandboxed guest environment. Filesystem operations will fail silently or throw `Operation not permitted (os error 63)` (`ENOTCAPABLE`) unless the appropriate WASI capabilities are requested and approved.

### Metadata Request
In `src/lib.rs`, the plugin metadata MUST request the scoped filesystem permissions:
```rust
fn metadata(&self) -> PluginMetadata {
    PluginMetadata {
        name: "SpeedControl".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        ...
        permissions: vec![
            "fs.read.data".into(),
            "fs.write.data".into(),
        ],
    }
}
```

### Pre-approving Hash in `permission_cache.json`
To bypass the interactive console approval prompt `[y/N]` when starting the server:
1. Calculate the SHA-256 hash of the compiled `SpeedControl.wasm`.
2. Add the hash entry as `approved: true` in the host's `plugins/permission_cache.json`:
   ```json
   {
     "entries": {
       "<wasm-sha256-hash-lowercase>": {
         "permissions_requested": [
           "fs.read.data",
           "fs.write.data"
         ],
         "approved": true
       }
     }
   }
   ```

---

## 3. Critical Windows & WASI Gotchas

### ⚠️ Gotcha A: Windows Backslash Path Separation in WASI Preopens
- **Problem**: On Windows hosts, `context.get_data_folder()` returns a path with backslashes (e.g., `"plugins\SpeedControl"`).
- **WASI Path Matching**: Wasmtime matches the guest's path prefix *exactly* against the preopened directory string. In the WASM guest's Unix-like environment, backslashes (`\`) are treated as normal characters, not separators.
- **Gotcha**: If you normalize the path string in the guest to use forward slashes (e.g. `"plugins/SpeedControl"`), it will **fail to prefix-match** the preopened directory `"plugins\SpeedControl"`, resulting in `No such file or directory (os error 44)` or `ENOTCAPABLE`.
- **Solution**: Keep the raw, un-normalized path returned by `get_data_folder()` as-is. Formatting paths as `format!("{}/config.json", self.data_folder)` is safe because the prefix matches `"plugins\SpeedControl"`, and the trailing slash is correctly matched by the OS after stripping the preopen prefix.

### ⚠️ Gotcha B: Byte Order Mark (BOM) in Config Files
- **Problem**: Writing/updating the server's `permission_cache.json` using PowerShell commands (e.g., `Out-File` or `>`) can inject a UTF-8 BOM.
- **Gotcha**: The Pumpkin server's JSON deserializer (`serde_json`) does not ignore BOM characters, leading to silent deserialization errors and forcing the server to ignore the cache and prompt for permissions.
- **Solution**: Ensure files are saved as standard UTF-8 (without BOM). Let the server auto-save it on first approval, or use scripts that write raw UTF-8 bytes.

---

## 4. Useful Server Management Commands

When the server is running, administrators can manage plugins using the following subcommands:

- **Unload Plugin**: `/plugin unload SpeedControl`
- **Load Plugin**: `/plugin load SpeedControl.wasm`
- **Enable Hot-Reload**: `/plugin hotreload enable` *(monitors the plugins folder and automatically reloads the WASM when modified)*
- **List Loaded Plugins**: `/plugin list`
- **Reload SpeedConfig manually**: `/speed reload` *(re-reads `config.json` inside the plugin's data folder on the fly)*
