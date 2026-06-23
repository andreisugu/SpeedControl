#!/usr/bin/env bash
# build.sh - Bash script to build, test, and deploy the SpeedControl plugin for Linux/macOS devs.
set -e

# Default parameters
CLEAN=false
DEV=false
LINT=false
TEST=false
CI=false
FEATURES=""
DEPLOY_DIR="$HOME/Desktop/Pumpkin/plugins"

# Helper for showing usage
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --clean, -c        Clean target folders before building"
    echo "  --dev, -d          Build using development/debug profile (default is release)"
    echo "  --lint, -l         Run formatting and clippy checks"
    echo "  --test, -t         Run unit tests natively"
    echo "  --ci               Run all GitHub Actions quality gates and compile on success"
    echo "  --features, -f VAL Specify Cargo features to build with (comma-separated)"
    echo "  --deploy-dir, -o   Deploy directory path (default: ~/Desktop/Pumpkin/plugins)"
    echo "  --help, -h         Show this help message"
    exit 1
}

# Parse options
while [[ "$#" -gt 0 ]]; do
    case "$1" in
        --clean|-c) CLEAN=true; shift ;;
        --dev|-d) DEV=true; shift ;;
        --lint|-l) LINT=true; shift ;;
        --test|-t) TEST=true; shift ;;
        --ci) CI=true; shift ;;
        --features|-f) FEATURES="$2"; shift 2 ;;
        --deploy-dir|-o) DEPLOY_DIR="$2"; shift 2 ;;
        --help|-h) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

FEATURE_FLAGS=()
if [ -n "$FEATURES" ]; then
    echo -e "\033[0;36mCompiling with features: $FEATURES\033[0m"
    FEATURE_FLAGS=("--features" "$FEATURES")
fi

if [ "$CI" = true ]; then
    echo -e "\033[0;32mExecuting FULL CI WORKFLOW verification...\033[0m"

    # 1. Formatting
    echo -e "\033[0;36m1. Running cargo fmt check...\033[0m"
    cargo fmt --all -- --check

    # 2. Clippy (All features, warnings as errors)
    echo -e "\033[0;36m2. Running clippy verification...\033[0m"
    cargo clippy --all-targets --all-features -- -D warnings

    # 3. Cargo Deny (Fail if missing or fails)
    echo -e "\033[0;36m3. Running cargo deny check...\033[0m"
    if ! command -v cargo-deny &> /dev/null; then
        echo -e "\033[0;31mError: cargo-deny is required for CI verification! Please install it: cargo install --locked cargo-deny\033[0m"
        exit 1
    fi
    cargo deny check

    # 4. Cargo Audit (Fail if missing or fails)
    echo -e "\033[0;36m4. Running cargo audit...\033[0m"
    if ! command -v cargo-audit &> /dev/null; then
        echo -e "\033[0;31mError: cargo-audit is required for CI verification! Please install it: cargo install cargo-audit --locked\033[0m"
        exit 1
    fi
    cargo audit

    # Resolve host
    RUSTC_HOST=$(rustc -vV | grep "host:" | cut -d ' ' -f 2)
    echo -e "\033[0;36mDetected native host target: $RUSTC_HOST\033[0m"

    # 5. Native unit tests (No features)
    echo -e "\033[0;36m5. Running unit tests with no features...\033[0m"
    cargo test --no-default-features --target "$RUSTC_HOST"

    # 6. Native unit tests (All features)
    echo -e "\033[0;36m6. Running unit tests with all features...\033[0m"
    cargo test --all-features --target "$RUSTC_HOST"

    echo -e "\033[0;32mVerification succeeded! Building release WASM artifact...\033[0m"
    
    # Overwrite options for build phase
    DEV=false
    CLEAN=false
    LINT=false
    TEST=false
    FEATURES=""
    FEATURE_FLAGS=()
fi

# 1. Clean
if [ "$CLEAN" = true ]; then
    echo -e "\033[0;36mCleaning target folders...\033[0m"
    cargo clean
    for folder in target_build build_check .cargo/target; do
        if [ -d "$folder" ]; then
            echo -e "\033[0;33mRemoving duplicate folder: $folder\033[0m"
            rm -rf "$folder"
        fi
    done
fi

# 2. Lint
if [ "$LINT" = true ]; then
    echo -e "\033[0;36mRunning code formatting and clippy lint checks...\033[0m"
    echo -e "\033[0;36mRunning cargo fmt...\033[0m"
    cargo fmt --all -- --check

    echo -e "\033[0;36mRunning clippy...\033[0m"
    cargo clippy --all-targets "${FEATURE_FLAGS[@]}" -- -D warnings

    # Local cargo-deny and cargo-audit if installed
    if command -v cargo-deny &> /dev/null; then
        echo -e "\033[0;36mRunning cargo deny...\033[0m"
        cargo deny check "${FEATURE_FLAGS[@]}"
    else
        echo -e "\033[0;33mcargo-deny is not installed. Skipping local license/source audits.\033[0m"
        echo -e "\033[0;33mTo install: cargo install --locked cargo-deny\033[0m"
    fi

    if command -v cargo-audit &> /dev/null; then
        echo -e "\033[0;36mRunning cargo audit...\033[0m"
        cargo audit
    else
        echo -e "\033[0;33mcargo-audit is not installed. Skipping local security audits.\033[0m"
        echo -e "\033[0;33mTo install: cargo install cargo-audit --locked\033[0m"
    fi
fi

# 3. Test
if [ "$TEST" = true ]; then
    echo -e "\033[0;36mRunning unit tests natively...\033[0m"
    RUSTC_HOST=$(rustc -vV | grep "host:" | cut -d ' ' -f 2)
    echo -e "\033[0;36mDetected native host target: $RUSTC_HOST\033[0m"
    cargo test --target "$RUSTC_HOST" "${FEATURE_FLAGS[@]}"
fi

# 4. Profile flags
PROFILE="release"
BUILD_FLAGS=("--release")
if [ "$DEV" = true ]; then
    PROFILE="debug"
    BUILD_FLAGS=()
fi

# 5. Build
echo -e "\033[0;36mCompiling plugin (profile: $PROFILE)...\033[0m"
cargo build "${BUILD_FLAGS[@]}" "${FEATURE_FLAGS[@]}"

# 6. Find compiled WASM
WASM_NAME="SpeedControl.wasm"
WASM_PATH="target/wasm32-wasip2/$PROFILE/$WASM_NAME"

if [ ! -f "$WASM_PATH" ]; then
    echo -e "\033[0;31mError: Compiled WASM not found at $WASM_PATH\033[0m"
    exit 1
fi

# 7. Deploy
if [ -n "$DEPLOY_DIR" ]; then
    # Expand tilde ~ if present
    DEPLOY_DIR="${DEPLOY_DIR/#\~/$HOME}"
    
    mkdir -p "$DEPLOY_DIR"
    DEST_PATH="$DEPLOY_DIR/$WASM_NAME"
    echo -e "\033[0;36mDeploying $WASM_PATH to $DEST_PATH...\033[0m"
    cp "$WASM_PATH" "$DEST_PATH"

    # Pre-approve permissions in permission_cache.json
    CACHE_PATH="$DEPLOY_DIR/permission_cache.json"
    if [ -f "$CACHE_PATH" ]; then
        echo -e "\033[0;36mPre-approving permissions in permission_cache.json...\033[0m"
        
        # Calculate SHA256 of the deployed file
        if command -v sha256sum &> /dev/null; then
            HASH=$(sha256sum "$DEST_PATH" | cut -d ' ' -f 1)
        elif command -v shasum &> /dev/null; then
            HASH=$(shasum -a 256 "$DEST_PATH" | cut -d ' ' -f 1)
        else
            HASH=""
            echo -e "\033[0;33mWarning: Neither sha256sum nor shasum available. Skipping permission pre-approval.\033[0m"
        fi

        if [ -n "$HASH" ]; then
            HASH=$(echo "$HASH" | tr '[:upper:]' '[:lower:]')
            
            # Using python to securely read/write JSON without dependencies like jq
            python3 -c "
import json, os
cache_path = '$CACHE_PATH'
hash_val = '$HASH'
with open(cache_path, 'r', encoding='utf-8') as f:
    data = json.load(f)

if 'entries' not in data:
    data['entries'] = {}

data['entries'][hash_val] = {
    'permissions_requested': ['fs.read.data', 'fs.write.data'],
    'approved': True
}

with open(cache_path, 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2)
"
            echo -e "\033[0;32mPre-approved WASM hash: $HASH\033[0m"
        fi
    fi
fi

echo -e "\033[0;32mBuild and deployment completed successfully!\033[0m"
