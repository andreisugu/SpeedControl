<#
.SYNOPSIS
    Builds and deploys the SpeedControl WebAssembly plugin.
.PARAMETER Clean
    If specified, cleans all target folders before building.
.PARAMETER Dev
    If specified, compiles a debug build instead of a release build.
.PARAMETER DeployDir
    Optional custom deployment directory. Defaults to "C:\Users\RestlessGlass\Desktop\Pumpkin\plugins".
#>
param(
    [switch]$Clean,
    [switch]$Dev,
    [switch]$Lint,
    [switch]$Test,
    [string]$DeployDir = "C:\Users\RestlessGlass\Desktop\Pumpkin\plugins"
)

# 1. Clean target folders if requested
if ($Clean) {
    Write-Host "Cleaning target folders..." -ForegroundColor Cyan
    cargo clean
    
    # Remove duplicate target folders
    $duplicates = @("target_build", "build_check", ".cargo/target")
    foreach ($folder in $duplicates) {
        if (Test-Path $folder) {
            Write-Host "Removing duplicate folder: $folder" -ForegroundColor Yellow
            Remove-Item -Path $folder -Recurse -Force
        }
    }
}

# 1b. Run Lint Checks if requested
if ($Lint) {
    Write-Host "Running code formatting and clippy lint checks..." -ForegroundColor Cyan
    Write-Host "Running cargo fmt..." -ForegroundColor Cyan
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Formatting check failed! Run 'cargo fmt' to resolve."
        exit $LASTEXITCODE
    }

    Write-Host "Running clippy..." -ForegroundColor Cyan
    cargo clippy --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Clippy lint checks failed!"
        exit $LASTEXITCODE
    }
}

# 1c. Run Tests if requested
if ($Test) {
    Write-Host "Running unit tests natively..." -ForegroundColor Cyan
    # Dynamically resolve native host target
    $rustcHost = (rustc -vV | Select-String "host:").Line.Split(" ")[1].Trim()
    Write-Host "Detected native host target: $rustcHost" -ForegroundColor Cyan
    cargo test --target $rustcHost
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Unit tests failed!"
        exit $LASTEXITCODE
    }
}

# 2. Determine profile
$profile = "release"
$buildFlags = @("--release")
if ($Dev) {
    $profile = "debug"
    $buildFlags = @()
}

# 3. Build the plugin
Write-Host "Compiling plugin (profile: $profile)..." -ForegroundColor Cyan
cargo build @buildFlags
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed!"
    exit $LASTEXITCODE
}

# 4. Find the compiled WASM
$wasmName = "SpeedControl.wasm"
$wasmPath = "target/wasm32-wasip2/$profile/$wasmName"

if (-not (Test-Path $wasmPath)) {
    Write-Error "Compiled WASM not found at $wasmPath"
    exit 1
}

# 5. Deploy to static plugin directory
if ($DeployDir) {
    if (-not (Test-Path $DeployDir)) {
        Write-Host "Creating deployment directory: $DeployDir" -ForegroundColor Yellow
        New-Item -ItemType Directory -Force -Path $DeployDir | Out-Null
    }

    $destPath = Join-Path $DeployDir $wasmName
    Write-Host "Deploying $wasmPath to $destPath..." -ForegroundColor Cyan
    Copy-Item -Path $wasmPath -Destination $destPath -Force

    # 6. Pre-approve permissions in permission_cache.json
    $cachePath = Join-Path $DeployDir "permission_cache.json"
    if (Test-Path $cachePath) {
        Write-Host "Pre-approving permissions in permission_cache.json..." -ForegroundColor Cyan
        
        # Calculate SHA256 of the deployed file
        $hash = (Get-FileHash -Path $destPath -Algorithm SHA256).Hash.ToLower()
        
        # Load permission cache
        $cache = Get-Content -Path $cachePath -Raw | ConvertFrom-Json
        
        # Add entry if it doesn't exist or is different
        if (-not $cache.entries) {
            $cache = New-Object PSObject -Property @{ entries = New-Object PSObject }
        }
        
        $entryValue = @{
            permissions_requested = @("fs.read.data", "fs.write.data")
            approved = $true
        }

        $entriesObj = $cache.entries
        if ($entriesObj.PSObject.Properties[$hash]) {
            $entriesObj.PSObject.Properties[$hash].Value = $entryValue
        } else {
            $entriesObj.PSObject.Properties.Add((New-Object System.Management.Automation.PSNoteProperty($hash, $entryValue)))
        }
        
        $jsonText = $cache | ConvertTo-Json -Depth 5
        $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
        [System.IO.File]::WriteAllText($cachePath, $jsonText, $utf8NoBom)
        Write-Host "Pre-approved WASM hash: $hash" -ForegroundColor Green
    }
}

Write-Host "Build and deployment completed successfully!" -ForegroundColor Green
