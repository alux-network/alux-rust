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

# Check that each crate packages cleanly.
package:
    cargo package --list --workspace --exclude http-providers > /dev/null

# Run the whole gate, in the order CI runs it.
ci: fmt build clippy doc test package

# Bumps each `package:level` (patch, minor, or major) and the requirement `[workspace.dependencies]`
# states for it, commits every bump in one `chore: release`, publishes the crates in dependency order,
# then tags each `<crate>-v<version>` and pushes the commit and the tags. Without `--execute` it is a
# dry run that changes nothing, so its later steps read the versions from before the bump.
#   just release alux-sdk:minor alux-ext:patch alux-http-poem:minor
#   just release --execute alux-sdk:minor alux-ext:patch alux-http-poem:minor
# Release crates, each by its own level, in one commit; a dry run without `--execute`; order does not matter.
release +releases:
    #!/usr/bin/env sh
    set -eu
    execute=""
    packages=""
    for release in {{releases}}; do
        if [ "$release" = "--execute" ]; then execute="--execute"; fi
    done
    for release in {{releases}}; do
        [ "$release" = "--execute" ] && continue
        cargo release version "${release##*:}" $execute --no-confirm --package "${release%%:*}"
        packages="$packages --package ${release%%:*}"
    done
    cargo release commit $execute --no-confirm
    cargo release publish $execute --no-confirm $packages
    cargo release tag $execute --no-confirm $packages
    cargo release push $execute --no-confirm $packages

# Remove build artifacts.
clean:
    cargo clean
