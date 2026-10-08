#!/bin/sh
set -eu
# Source mount is read-only; all build and consumer files stay in the container.
export CARGO_TARGET_DIR=/tmp/rocsteady-target
cargo test --locked
cargo test --locked --all-features --all-targets
cargo test --locked --all-features --doc
cargo run --locked --example managed_extensions
cargo run --locked --features simulator --example authenticated_setup
cargo run --locked --features simulator --example simulator
cargo package --locked --allow-dirty --all-features
consumer=/tmp/rocsteady-consumer
package_version=$(cargo pkgid --offline | sed 's/.*@//')
mkdir -p "$consumer/src"
cat > "$consumer/Cargo.toml" <<EOF
[package]
name = "rocsteady-release-consumer"
version = "0.0.0"
edition = "2024"
[dependencies]
rocsteady = { path = "/tmp/rocsteady-target/package/rocsteady-$package_version", features = ["serial", "simulator"] }
etherbird = { version = "0.3.1", features = ["pool"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
EOF
cp scripts/package-consumer.rs "$consumer/src/main.rs"
cp Cargo.lock "$consumer/Cargo.lock"
cargo run --offline --manifest-path "$consumer/Cargo.toml"
