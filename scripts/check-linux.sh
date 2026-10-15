#!/bin/sh
set -eu
# Source mount is read-only; all build and consumer files stay in the container.
export CARGO_TARGET_DIR=/tmp/rocsteady-target
cargo test --locked
cargo test --locked --all-features --all-targets
cargo test --locked --all-features --doc
cargo run --locked --example managed_extensions
cargo run --locked --example schema_profiles
cargo run --locked --example discovery_catalog
cargo run --locked --example log_retrieval
cargo run --locked --example history_metadata
cargo run --locked --example history
cargo run --locked --features simulator --example authenticated_setup
cargo run --locked --features simulator --example simulator
cargo package --locked --allow-dirty --all-features
consumer=/tmp/rocsteady-consumer
package_id=$(cargo pkgid --offline)
# Cargo emits either #version or #name@version for a local package.
package_version=${package_id##*[@#]}
mkdir -p "$consumer/src"
# The package version is a Cargo-validated semver and the container path is fixed.
sed "s|@PACKAGE_PATH@|\"/tmp/rocsteady-target/package/rocsteady-$package_version\"|" \
    scripts/package-consumer.toml.in > "$consumer/Cargo.toml"
cp scripts/package-consumer.rs "$consumer/src/main.rs"
cp "/tmp/rocsteady-target/package/rocsteady-$package_version/Cargo.lock" "$consumer/Cargo.lock"
cargo run --offline --manifest-path "$consumer/Cargo.toml"
