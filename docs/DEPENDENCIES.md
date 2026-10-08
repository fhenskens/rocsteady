# Dependency and publication audit

Reviewed 2026-10-09 from `cargo metadata --locked --all-features` and the
registry source manifests for the tracked Cargo.lock, including target-specific
dependencies. This is a license inventory and package review, not a security
advisory audit. No dependency source is vendored in the rocsteady package.

Direct dependencies: Etherbird 0.3.1 (Apache-2.0), Tokio 1.53.2 (MIT),
thiserror 2.0.21 (MIT OR Apache-2.0), optional tokio-serial 5.5.0 (MIT).
Serial adds serialport 4.10.1 (MPL-2.0); retain its original source/license
notices if redistributing that dependency. unescaper permits MIT as an alternative
to GPL-3.0-only; the MIT branch is selected for this release inventory.
unicode-ident includes Unicode-3.0 in addition to its MIT/Apache alternatives.
Actual license files for serialport, unescaper and unicode-ident were inspected.
The optional serial feature is therefore not accurately described as entirely MIT/Apache.

No missing declared license was found. Dependency upgrades must refresh this
inventory and repeat lifecycle, MSRV, platform and package consumer checks.

| Dependency | Locked version | Declared license |
| --- | --- | --- |
| async-trait | 0.1.92 | MIT OR Apache-2.0 |
| bitflags | 1.3.2 | MIT/Apache-2.0 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| bytes | 1.12.1 | MIT |
| cfg_aliases | 0.2.2 | MIT |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| core-foundation | 0.10.1 | MIT OR Apache-2.0 |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 |
| etherbird | 0.3.1 | Apache-2.0 |
| futures-core | 0.3.34 | MIT OR Apache-2.0 |
| futures-sink | 0.3.34 | MIT OR Apache-2.0 |
| io-kit-sys | 0.4.1 | MIT / Apache-2.0 |
| libc | 0.2.190 | MIT OR Apache-2.0 |
| log | 0.4.34 | MIT OR Apache-2.0 |
| mach2 | 0.4.3 | BSD-2-Clause OR MIT OR Apache-2.0 |
| mio | 1.2.4 | MIT |
| mio-serial | 5.0.7 | MIT |
| nix | 0.26.4 | MIT |
| nix | 0.31.3 | MIT |
| once_cell | 1.21.4 | MIT OR Apache-2.0 |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 |
| serialport | 4.10.1 | MPL-2.0 |
| socket2 | 0.6.5 | MIT OR Apache-2.0 |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.6 | MIT OR Apache-2.0 |
| thiserror | 2.0.21 | MIT OR Apache-2.0 |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 |
| tokio | 1.53.2 | MIT |
| tokio-macros | 2.7.2 | MIT |
| tokio-serial | 5.5.0 | MIT |
| tracing | 0.1.44 | MIT |
| tracing-attributes | 0.1.31 | MIT |
| tracing-core | 0.1.36 | MIT |
| unescaper | 0.1.10 | MIT OR GPL-3.0-only |
| unicode-ident | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| windows_aarch64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_aarch64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_gnu | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_i686_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_gnu | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| windows-link | 0.2.1 | MIT OR Apache-2.0 |
| windows-sys | 0.52.0 | MIT OR Apache-2.0 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 |
| windows-targets | 0.52.6 | MIT OR Apache-2.0 |

Publication contents were inspected using `cargo package --list --locked --allow-dirty`.
The payload includes source, lockfile/manifests, tests, examples and fixture support,
README, Apache license, changelog and the selected user-facing guides/audit.
It excludes target output, local environment files, git internals, roadmaps, vendor
research, CI and developer scripts. Cargo-generated VCS metadata records dirty
candidate status; final publication requires a reviewed clean commit.

The README and usage include_str paths are present in the package. Verification
builds the extracted crate, and scripts compile/run an external consumer against
that extracted path with only public APIs. No sibling Etherbird checkout is used.
