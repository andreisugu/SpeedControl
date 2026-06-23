# Contributing to SpeedControl

Thank you for your interest in contributing to `SpeedControl`! We welcome all contributions, including bug reports, feature requests, documentation improvements, and pull requests.

## Development Workflow

To ensure a smooth contribution process, please follow these guidelines:

### 1. Build and Test Locally

We have provided all-in-one local coordination scripts to verify formatting, clippy lints, dependency licenses, vulnerabilities, and unit tests.

#### Windows (PowerShell)
Before committing, execute the full CI verification switch:
```powershell
.\build.ps1 -CI
```
This runs `cargo fmt`, `cargo clippy --all-features -- -D warnings`, `cargo deny check`, `cargo audit`, and all unit tests under all feature combinations. It will compile the release binary only if everything is green.

#### Linux & macOS (Bash)
Similarly, on Unix environments, execute:
```bash
./build.sh --ci
```

### 2. Formatting & Clippy Lints
We enforce strict style conventions:
* Run `cargo fmt` to automatically format all source files.
* Make sure your code compiles with zero Clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`).

### 3. Licensing
By contributing to this project, you agree that your contributions will be dual-licensed under the terms of both the **MIT License** and the **Apache License, Version 2.0**.
