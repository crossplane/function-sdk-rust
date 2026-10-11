//! A Rust SDK for writing Crossplane composition functions.
//!
//! Composition functions are gRPC servers implementing the
//! [`FunctionRunnerService`] defined by Crossplane's
//! `apiextensions.fn.proto.v1` API. This SDK provides the generated protocol
//! types, a spec-compliant server runtime, and helpers for working with
//! requests and responses.
//!
//! The most important protocol rule: desired state is a fully specified
//! server-side apply intent, not a merge patch. Build responses with
//! [`response::to`] so the desired state and context accumulated by earlier
//! pipeline steps are copied forward; anything left out is deleted from the
//! cluster.
//!
//! [`FunctionRunnerService`]: proto::v1::function_runner_service_server::FunctionRunnerService

mod metrics;

pub mod context;
pub mod logging;
pub mod proto;
pub mod request;
pub mod resource;
pub mod response;
pub mod server;

pub use server::{Args, Server, serve};

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
    #[error("cannot parse listen address: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),

    /// Neither --tls-certs-dir nor --insecure was supplied.
    #[error("no credentials were provided - supply --tls-certs-dir or use --insecure")]
    MissingTlsCertsDir,

    /// A TLS certificate or key could not be read.
    #[error("cannot read TLS certificate or key from {path}: {source}")]
    ReadCertificate {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    /// The gRPC server failed.
    #[error("gRPC server error: {0}")]
    Transport(#[from] tonic::transport::Error),

    /// The gRPC reflection service could not be built.
    #[error("cannot build gRPC reflection service: {0}")]
    Reflection(#[from] tonic_reflection::server::Error),

    /// The readiness future set with [`Server::ready`] failed.
    #[error("the function did not become ready: {0}")]
    NotReady(Box<dyn std::error::Error + Send + Sync>),
}
