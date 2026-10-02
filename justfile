set shell := ["bash", "-euo", "pipefail", "-c"]

# Nightly used only to emit rustdoc JSON for docs/sdk/ (the format is unstable). Bump it together
# with the `rustdoc-types` version in xtask/Cargo.toml; the rest of the workspace stays on the
# toolchain in rust-toolchain.toml.
docs_nightly := "nightly-2026-10-02"

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

# Run the live tests against the production APIs (read-only; needs network)
[group("test")]
test-live *args:
    cargo test --all-features --test live -- --ignored --nocapture {{ args }}

# Build docs with warnings as errors
[group("docs")]
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Regenerate docs/sdk/ (markdown API docs for LLMs) from rustdoc JSON
[group("docs")]
docs-md: _sdk-docs-json
    cargo xtask docs

# Fail if docs/sdk/ is out of date (needs the pinned nightly, so not part of `check`)
[group("docs")]
docs-md-check: _sdk-docs-json
    cargo xtask docs --check

# Emit rustdoc JSON for both crates with the pinned nightly into target/sdk-docs/
_sdk-docs-json:
    rustup toolchain install {{ docs_nightly }} --profile minimal --no-self-update
    cargo +{{ docs_nightly }} rustdoc -p marcasite-core --all-features --target-dir target/sdk-docs -- -Z unstable-options --output-format json
    cargo +{{ docs_nightly }} rustdoc -p marcasite --all-features --target-dir target/sdk-docs -- -Z unstable-options --output-format json

# Check dependency advisories, licences, bans and sources
[group("lint")]
deny:
    cargo deny check

# Run every check CI runs
check: fmt-check clippy test doc deny
