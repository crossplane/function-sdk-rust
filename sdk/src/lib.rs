//! A Rust SDK for writing Crossplane composition functions.
//!
//! Composition functions are gRPC servers implementing the
//! `FunctionRunnerService` defined by Crossplane's `apiextensions.fn.proto.v1`
//! API. This SDK provides the generated protocol types, a spec-compliant
//! server runtime, and helpers for working with requests and responses.
//!
//! The most important protocol rule: desired state is a fully specified
//! server-side apply intent, not a merge patch. Build responses with
//! [`response::to`] so the desired state and context accumulated by earlier
//! pipeline steps are copied forward; anything left out is deleted from the
//! cluster.
//!
//! # Features
//!
//! - `server` (enabled by default): the server side - `serve` and `Args`,
//!   `logging`, the generated tonic client and server under `proto::v1`, and
//!   `proto::FILE_DESCRIPTOR_SET` - with the dependencies behind it (tokio,
//!   tonic and its TLS stack, clap, prometheus-client). Without it, with
//!   `default-features = false`, the crate is the protocol messages and the
//!   [`request`], [`response`] and [`resource`] helpers, which also build for
//!   WebAssembly targets: what a function compiled to a WebAssembly module
//!   depends on.

#[cfg(feature = "server")]
mod metrics;

pub mod context;
#[cfg(feature = "server")]
pub mod logging;
pub mod proto;
pub mod request;
pub mod resource;
pub mod response;
#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "server")]
pub use server::{Args, serve};

/// Errors returned by the SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A JSON conversion failed.
    #[error("JSON conversion failed: {0}")]
    Json(#[from] serde_json::Error),

    /// A source serialized to a JSON value that is not an object.
    #[error("source must serialize to a JSON object")]
    NotAnObject,

    /// The request has no function input.
    #[error("the request has no function input")]
    MissingInput,

    /// The resource has no JSON representation to deserialize.
    #[error("the resource has no JSON representation")]
    MissingResource,

    /// The listen address could not be parsed.
    #[cfg(feature = "server")]
    #[error("cannot parse listen address: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),

    /// Neither --tls-certs-dir nor --insecure was supplied.
    #[cfg(feature = "server")]
    #[error("no credentials were provided - supply --tls-certs-dir or use --insecure")]
    MissingTlsCertsDir,

    /// A TLS certificate or key could not be read.
    #[cfg(feature = "server")]
    #[error("cannot read TLS certificate or key from {path}: {source}")]
    ReadCertificate {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    /// The gRPC server failed.
    #[cfg(feature = "server")]
    #[error("gRPC server error: {0}")]
    Transport(#[from] tonic::transport::Error),

    /// The gRPC reflection service could not be built.
    #[cfg(feature = "server")]
    #[error("cannot build gRPC reflection service: {0}")]
    Reflection(#[from] tonic_reflection::server::Error),
}
