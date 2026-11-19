# Agents

Instructions for AI agents working in this repository. Read
[CONTRIBUTING.md](CONTRIBUTING.md) before opening an issue or a pull request;
it describes the workflows these rules refer to.

## What this is

A Rust SDK for Crossplane composition functions: generated protocol types, a
gRPC server runtime, and request and response helpers. The
[README](README.md#layout) describes the layout.

## Checks

Run all of these before you call a change done. CI requires them.

```shell
cargo fmt --all --check
cargo clippy --workspace --tests -- -D warnings
cargo test --workspace
cargo run -p codegen   # needs protoc; must leave the generated files unchanged
```

## Rules

- Never edit `sdk/src/generated` or `sdk/tests/generated` by hand. Change the
  proto or `codegen/` and regenerate.
- Every message keeps unknown fields: `codegen/src/messages.rs` writes the
  `prost::Message` impls in place of prost's derive. When codegen refuses a
  field kind, or a test in `sdk/tests/generated_messages.rs` fails, follow
  CONTRIBUTING.md.
- A new public field on a message struct is a breaking change. Say so in the
  commit and the pull request.
- Don't change the crate version outside a release pull request.
- Keep the SDK's behaviour aligned with function-sdk-go, and record a
  deliberate difference in the README section on differences.
- Sign off and sign every commit, and write Conventional Commits subjects.
- One logical change per pull request. Don't mix in unrelated fixes.
- Don't publish, tag or run the `Publish` workflow.
