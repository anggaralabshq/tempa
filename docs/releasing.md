# Releasing tempa

A release is a version tag. `.github/workflows/release.yml` does the rest.

## One-time setup

1. **crates.io**: create an API token (scope `publish-new`, then `publish-update`) and save it as the repo secret `CARGO_REGISTRY_TOKEN`. Without it the crates.io job skips.
2. **Homebrew**: create a fine-grained personal access token with *Contents: read and write* on `anggaralabshq/homebrew-tap` only, and save it as the repo secret `HOMEBREW_TAP_TOKEN`. Without it the Homebrew job skips.
3. The tap and this repo must be public before `brew install` works for anyone else: the formula downloads the release archives anonymously.

## Each release

1. Set `version` in the workspace `Cargo.toml` (both crates inherit it) and in `crates/tempa-cli/Cargo.toml`'s `tempa-core` dependency.
2. Run the checks in `HARNESS.md`; merge to `main`.
3. Tag and push: `git tag v1.2.3 && git push origin v1.2.3`.
4. The workflow builds `tempa-v1.2.3-<target>` archives (macOS arm64/x86_64, Linux arm64/x86_64 musl, Windows x86_64) with `.sha256` files, attaches them to the GitHub Release, publishes `tempa-core` then `tempa-cli` to crates.io, and writes `Formula/tempa.rb` in the tap (`scripts/homebrew-formula.sh`).
5. Check: `brew install anggaralabshq/tap/tempa && tempa --version`.

crates.io never takes a version twice: if publishing fails after `tempa-core` went up, bump the patch version and tag again.
