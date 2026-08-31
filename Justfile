# Check formatting without rewriting any file.
fmt:
    cargo fmt --all -- --check

# Compile every crate, target, and test with all features enabled.
build:
    cargo build --workspace --all-features --all-targets

# Run every test; doc tests run separately because Nextest does not support them.
test:
    cargo nextest run --workspace --all-features --no-fail-fast
    cargo test --workspace --all-features --doc

# Lint every target, treating warnings as errors.
clippy:
    cargo clippy --workspace --all-features --all-targets -- -D warnings

# Build the docs, denying rustdoc warnings such as broken intra-doc links.
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps

# Check that each crate packages cleanly, listed in publication order.
package:
    cargo package --list -p alux-ext-macros > /dev/null
    cargo package --list -p alux-sdk-macros > /dev/null
    cargo package --list -p alux-ext > /dev/null
    cargo package --list -p alux-http > /dev/null
    cargo package --list -p alux-jsonrpc > /dev/null
    cargo package --list -p alux-traversable > /dev/null
    cargo package --list -p alux-sdk > /dev/null
    cargo package --list -p alux-shape-macros > /dev/null
    cargo package --list -p alux-shape > /dev/null
    cargo package --list -p alux-shape-json > /dev/null
    cargo package --list -p alux-shape-text > /dev/null
    cargo package --list -p alux-shape-typescript > /dev/null
    cargo package --list -p alux-shape-rust > /dev/null
    cargo package --list -p alux-shape-term > /dev/null
    cargo package --list -p alux-shape-morph > /dev/null
    cargo package --list -p alux-http-text > /dev/null
    cargo package --list -p alux-http-poem > /dev/null
    cargo package --list -p alux-jsonrpc-direct > /dev/null
    cargo package --list -p alux-jsonrpc-jsonrpsee > /dev/null
    cargo package --list -p alux-tokio > /dev/null
    cargo package --list -p alux-jsonrpc-typescript > /dev/null

# Run the whole gate, in the order CI runs it.
ci: fmt build clippy doc test package

# Bump and publish every crate in the workspace; level is patch, minor, or major.
release level:
    cargo release --workspace {{level}} --execute

# Bump and publish one crate; level is patch, minor, or major.
release-crate package level:
    cargo release --package {{package}} {{level}} --execute

# Remove build artifacts.
clean:
    cargo clean
