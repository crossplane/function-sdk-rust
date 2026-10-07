# function-sdk-rust

A Rust SDK for writing [Crossplane](https://www.crossplane.io)
[composition functions](https://docs.crossplane.io/latest/composition/compositions/).

A composition function is a gRPC server implementing the
`FunctionRunnerService` defined by Crossplane's `apiextensions.fn.proto.v1`
API. This crate provides:

- `proto::v1` - generated protocol types with protojson serde support, plus,
  with the `server` feature, the tonic gRPC server and client.
- `server` - a function-spec-compliant runtime: mTLS or insecure serving,
  gRPC server reflection, the gRPC health service, graceful shutdown, and
  the standard CLI arguments.
- `request`, `response`, `resource` - helpers for typed function input,
  context keys, required resources and schemas, credentials, capabilities,
  results, conditions, readiness, and desired resource updates from any
  `serde::Serialize` source.
- `logging` - JSON-lines logging with a human-readable debug mode.

The `server` feature, on by default, carries the server side: `server`,
`logging`, the tonic client and server, and the dependencies behind them.
With `default-features = false` the crate is the protocol types and the
`request`, `response` and `resource` helpers, which also build for
WebAssembly targets: what a function compiled to a WebAssembly module
depends on.

See the [repository](https://github.com/crossplane/function-sdk-rust) for a
complete example function.
