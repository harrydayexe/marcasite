set shell := ["bash", "-euo", "pipefail", "-c"]

# List available recipes
[private]
default:
    @just --list

# Check formatting
[group("lint")]
fmt-check:
    cargo fmt --all -- --check

# Format all code
[group("lint")]
fmt:
    cargo fmt --all

# Run clippy with warnings as errors
[group("lint")]
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests, forwarding extra args to cargo test
[group("test")]
test *args:
    cargo test --all-features {{ args }}

# Build docs with warnings as errors
[group("docs")]
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Check dependency advisories, licences, bans and sources
[group("lint")]
deny:
    cargo deny check

# Run every check CI runs
check: fmt-check clippy test doc deny
