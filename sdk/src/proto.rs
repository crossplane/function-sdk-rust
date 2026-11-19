//! Generated protobuf, gRPC, and protojson (serde) code for the
//! `apiextensions.fn.proto.v1` FunctionRunnerService API.
//!
//! Regenerate with `cargo run -p codegen` after changing the vendored proto.
//! Every message keeps the fields its proto does not know in
//! `unknown_fields`: codegen writes the `prost::Message` impls for that.

mod unknown;

pub use unknown::UnknownFields;

/// Types and the gRPC service for the v1 FunctionRunnerService API.
pub mod v1 {
    #![allow(clippy::all, clippy::pedantic)]
    include!("generated/v1.rs");
    include!("generated/v1.serde.rs");
}

/// Encoded FileDescriptorSet of the v1 API, used for gRPC server reflection.
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/fileset.bin");
