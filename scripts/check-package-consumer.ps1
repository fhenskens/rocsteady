# Run after cargo package: verifies only the publication payload supplies the API.
$ErrorActionPreference = 'Stop'
$workspacePath = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$buildRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $workspacePath 'target' }
$packageMetadata = cargo metadata --offline --no-deps --format-version 1 --manifest-path (Join-Path $workspacePath 'Cargo.toml') | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Could not read release version' }
$packageVersion = ($packageMetadata.packages | Where-Object name -eq 'rocsteady').version
$packagePath = (Resolve-Path (Join-Path $buildRoot "package/rocsteady-$packageVersion")).Path.Replace('\', '/')
$consumerPath = Join-Path $buildRoot 'release-consumer'
New-Item -ItemType Directory -Force (Join-Path $consumerPath 'src') | Out-Null
@"
[package]
name = "rocsteady-release-consumer"
version = "0.0.0"
edition = "2024"
[dependencies]
rocsteady = { path = "$packagePath", features = ["serial", "simulator"] }
etherbird = { version = "0.3.1", features = ["pool"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
"@ | Set-Content -Encoding UTF8 (Join-Path $consumerPath 'Cargo.toml')
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'package-consumer.rs') -Destination (Join-Path $consumerPath 'src/main.rs')
# Inherit the same dependency resolution as the candidate rather than silently upgrading.
Copy-Item -LiteralPath (Join-Path $packagePath 'Cargo.lock') -Destination (Join-Path $consumerPath 'Cargo.lock')
cargo run --offline --manifest-path (Join-Path $consumerPath 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { throw 'Packaged consumer failed' }
