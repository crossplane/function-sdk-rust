# Contributing

Thanks for your interest in contributing to `function-sdk-rust`!

This project follows the governance and community guidelines of the broader
[Crossplane project][governance]. See [OWNERS.md](OWNERS.md) for the current
maintainer(s) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for expected
community behavior.

## Getting started

See the [README](README.md#layout) for the repository layout and the
[README](README.md#writing-a-function) for an introduction to the SDK.

## Development

```shell
# Run tests.
cargo test

# Lint.
cargo clippy --workspace --tests

# Regenerate sdk/src/generated from the vendored proto (requires protoc).
cargo run -p codegen
```

The `State` and `Resource` messages are written by hand in
`sdk/src/proto/state.rs` so that they keep unknown fields. When an update of
the vendored proto changes either message, update that file to match; a test
fails until you do.

## Submitting changes

* Open an issue first for anything beyond a small fix, so we can agree on the
  approach before you invest time in it.
* Keep pull requests focused on a single logical change.
* Make sure `cargo test` and `cargo clippy --workspace --tests` pass.
* Reference any related issue in the pull request description.

## Reporting security issues

Please see [SECURITY.md](SECURITY.md) instead of opening a public issue.

[governance]: https://github.com/crossplane/crossplane/blob/main/GOVERNANCE.md
