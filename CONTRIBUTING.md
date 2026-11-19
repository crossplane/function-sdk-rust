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

You need a stable Rust toolchain. Regenerating code also needs `protoc` on
your `PATH`.

```shell
# Format.
cargo fmt --all

# Lint. CI fails on warnings.
cargo clippy --workspace --tests -- -D warnings

# Run tests.
cargo test --workspace

# Regenerate sdk/src/generated from the vendored proto.
cargo run -p codegen
```

CI runs four checks on every pull request, and all four must pass before it
can merge: `fmt`, `clippy`, `test` and `codegen-drift`. The last one runs
codegen and fails when the `.rs` files in `sdk/src/generated` or
`sdk/tests/generated` differ from what's checked in.

### Generated protocol code

Don't edit `sdk/src/generated` or `sdk/tests/generated` by hand. Change the
proto or `codegen/`, then run codegen and commit the result.

prost discards a field it doesn't know, so codegen replaces prost's `Message`
derive. `codegen/src/messages.rs` gives every message an `unknown_fields`
field and writes the impls the derive would have written, with the unknown
fields kept and encoded again. Codegen also writes the messages as prost
derives them to `sdk/tests/generated/prost_v1.rs`.
`sdk/tests/generated_messages.rs` compares the two for every message: the
bytes, the errors on malformed input, `Debug`, the defaults and the
accessors.

### Updating the vendored proto

`sdk/proto/v1/run_function.proto` is a copy of the file in
[crossplane/crossplane][proto]. A comment above its `syntax` line names the
release it was copied from. Renovate reads that comment and opens a pull
request when Crossplane publishes a newer release. That pull request changes
only the version in the comment, so Renovate doesn't merge it by itself.
Finish it on its branch:

1. Replace the file with the one from the release tag, and put the
   `Vendored from crossplane/crossplane` comment back above `syntax` with the
   new version.
2. Run `cargo run -p codegen` and commit the changes in `sdk/src/generated`
   and `sdk/tests/generated`.
3. Set each new field in the samples of `sdk/tests/generated_messages.rs`.
   A sample lists every field of its message, so that file doesn't compile
   until you do.
4. If codegen stops at a field kind it doesn't write yet, its error gives
   the field and the `#[prost(...)]` attribute. Add the kind to
   `codegen/src/messages.rs`, following what prost's derive generates for it.
5. If the update adds a message, `the_checks_cover_every_message_of_the_proto`
   fails. Add the message to `for_every_message` and to `sample` in
   `sdk/tests/generated_messages.rs`.
6. Run the checks above.

A new field on any message is a breaking change for this crate, because the
message structs are public and can be built with a struct literal. Say so in
the pull request so the next release gets the right version.

### Dependency updates

Renovate merges minor, patch and digest updates by itself once the checks
pass. Major updates and proto updates wait for a maintainer.

Once a week Renovate also refreshes `Cargo.lock` within the ranges the
manifests allow and merges that the same way. An indirect dependency has no
line in a manifest, so this refresh is how a fix in one arrives. The lock
file is the repository's own: a function that depends on the SDK resolves
its dependencies itself.

A new version of `prost-build`, `tonic-prost-build` or `pbjson-build` can
change the generated code, and its pull request then fails `codegen-drift`.
Check out the branch, run `cargo run -p codegen`, and push the result to the
same branch.

A new version of `prost` can change what its `Message` derive does. Its pull
request then fails in `sdk/tests/generated_messages.rs`, and
`codegen/src/messages.rs` has to follow the derive before it merges.

## Submitting changes

* Open an issue first for anything beyond a small fix, so we can agree on the
  approach before you invest time in it.
* Keep pull requests focused on a single logical change, and reference any
  related issue in the description.
* Write commit subjects as [Conventional Commits][conventional-commits], for
  example `fix(logging): honor RUST_LOG`. Mark a breaking change with `!`
  and a `BREAKING CHANGE:` footer.
* Sign off every commit with `git commit -s`. The DCO check fails without
  the `Signed-off-by` line.
* Sign your commits. The main branch only accepts verified signatures.
* Add or update tests for behaviour you change, and update the README when
  the public API or a documented behaviour changes.
* Don't change the crate version in a feature pull request.

## Releasing

Maintainers release from the main branch:

1. Decide the version. While the crate is 0.x, a breaking change raises the
   minor version and anything else raises the patch version.
2. Open a pull request that sets `version` in `sdk/Cargo.toml` and updates
   `Cargo.lock`, titled `chore(sdk): bump version to <version>`.
3. After it merges, run the `Publish` workflow with the version prefixed by
   `v`, for example `v0.5.0`. The workflow refuses a version that doesn't
   match `sdk/Cargo.toml`, publishes the crate to crates.io, and creates the
   GitHub release with generated notes.

## Reporting security issues

Please see [SECURITY.md](SECURITY.md) instead of opening a public issue.

[governance]: https://github.com/crossplane/crossplane/blob/main/GOVERNANCE.md
[proto]: https://github.com/crossplane/crossplane/tree/main/proto/fn/v1
[conventional-commits]: https://www.conventionalcommits.org
