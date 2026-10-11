//! A spec-compliant gRPC server runtime for composition functions.

use std::future::Future;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use prometheus_client::registry::Registry;
use tonic::transport::{Certificate, Identity, Server as GrpcServer, ServerTlsConfig};
use tonic::{Request, Response, Status};
use tonic_health::ServingStatus;
use tonic_health::server::HealthReporter;

use crate::Error;
use crate::proto::v1::function_runner_service_server::{
    FunctionRunnerService, FunctionRunnerServiceServer, SERVICE_NAME,
};
use crate::proto::v1::{RunFunctionRequest, RunFunctionResponse};

/// CLI arguments required by the Crossplane composition function spec.
#[derive(clap::Parser, Debug)]
#[command(version, about = "A Crossplane composition function")]
pub struct Args {
    /// Emit debug logs.
    #[arg(short, long, env = "DEBUG", default_value_t = false)]
    pub debug: bool,

    /// Address at which to listen for gRPC connections.
    #[arg(long, default_value = "0.0.0.0:9443")]
    pub address: String,

    /// Directory containing tls.crt, tls.key, and ca.crt; serve using mTLS.
    #[arg(long, env = "TLS_SERVER_CERTS_DIR")]
    pub tls_certs_dir: Option<PathBuf>,

    /// Run without mTLS credentials. If set, --tls-certs-dir is ignored.
    #[arg(long, default_value_t = false)]
    pub insecure: bool,

    /// Maximum size in bytes of gRPC messages the function accepts.
    /// Defaults to the gRPC default of 4MB.
    #[arg(long)]
    pub max_recv_message_size: Option<usize>,

    /// Address at which to serve metrics at /metrics (OpenMetrics 1.0, or
    /// the classic Prometheus text format for an Accept header that asks
    /// for it) - the Go SDK's default; empty disables them.
    #[arg(long, env = "METRICS_ADDRESS", default_value = ":8080")]
    pub metrics_address: String,
}

/// Starts a gRPC server and serves RunFunctionRequests until SIGTERM or
/// SIGINT, then shuts down gracefully.
///
/// Serves with mTLS from `--tls-certs-dir` (tls.crt and tls.key must be the
/// function's PEM-encoded certificate and key; ca.crt a PEM-encoded CA used
/// to authenticate Crossplane) unless `--insecure` is set. gRPC server
/// reflection is enabled for both the v1 and v1alpha reflection APIs, and
/// the gRPC health service reports the function as serving.
///
/// Unless `--metrics-address` is empty, the gRPC server metrics the Go SDK
/// serves (`grpc_server_started_total` and friends, same names and labels)
/// are served there at /metrics - OpenMetrics 1.0 as the main format, the
/// classic Prometheus text format for scrapers that ask for it.
///
/// For options beyond the command-line `args`, use [`Server`].
pub async fn serve<F: FunctionRunnerService>(function: F, args: &Args) -> Result<(), Error> {
    Server::new(function, args).serve().await
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type ReadyFuture = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send>>;

/// A function server with options beyond the command-line [`Args`]. Create
/// it with [`Server::new`], chain the options, then call [`Server::serve`];
/// with no options it serves exactly like [`serve`]. New options arrive as
/// new methods, so code that builds a server keeps compiling.
///
/// ```no_run
/// # use function_sdk_rust::proto::v1::function_runner_service_server::FunctionRunnerService;
/// # async fn run(function: impl FunctionRunnerService, args: &function_sdk_rust::Args) -> Result<(), function_sdk_rust::Error> {
/// function_sdk_rust::Server::new(function, args)
///     .ready(async { Ok::<(), std::io::Error>(()) })
///     .serve()
///     .await
/// # }
/// ```
pub struct Server<'a, F> {
    function: F,
    args: &'a Args,
    ready: Option<ReadyFuture>,
    metrics_registry: Option<Registry>,
}

impl<'a, F: FunctionRunnerService> Server<'a, F> {
    /// Creates a server for `function`, configured by the command-line `args`.
    pub fn new(function: F, args: &'a Args) -> Self {
        Self {
            function,
            args,
            ready: None,
            metrics_registry: None,
        }
    }

    /// Holds the function back until `ready` completes, for a function that
    /// must finish loading before it can run. The server listens at once,
    /// but reports NOT_SERVING on the gRPC health service and answers
    /// RunFunction with UNAVAILABLE. Ok flips the function to SERVING; an
    /// error or a panic stops the server, and [`Server::serve`] returns it
    /// as [`Error::NotReady`].
    pub fn ready<R, E>(mut self, ready: R) -> Self
    where
        R: Future<Output = Result<(), E>> + Send + 'static,
        E: Into<BoxError>,
    {
        self.ready = Some(Box::pin(async move { ready.await.map_err(Into::into) }));
        self
    }

    /// Serves `registry` at /metrics, like function-sdk-go's
    /// `WithMetricsRegistry`: register the function's own series in it
    /// first, and the SDK adds its `grpc_server_*` series at the root. With
    /// `--metrics-address` empty nothing is served.
    ///
    /// Build the registry and its metrics with a `prometheus-client` version
    /// that is semver-compatible with the SDK's own dependency on it (while
    /// prometheus-client is 0.x, the same minor version). Cargo then resolves
    /// both to one copy of the crate; a registry from an incompatible version
    /// is a different type and does not compile here.
    pub fn metrics_registry(mut self, registry: Registry) -> Self {
        self.metrics_registry = Some(registry);
        self
    }

    /// Serves RunFunctionRequests until SIGTERM or SIGINT, then shuts down
    /// gracefully. See [`serve`] for what is served.
    pub async fn serve(self) -> Result<(), Error> {
        let Server {
            function,
            args,
            ready,
            metrics_registry,
        } = self;
        let address: SocketAddr = args.address.parse()?;

        let mut builder = GrpcServer::builder();
        if !args.insecure {
            let dir = args
                .tls_certs_dir
                .as_deref()
                .ok_or(Error::MissingTlsCertsDir)?;
            builder = builder.tls_config(tls_config(dir)?)?;
        }

        let is_ready = Arc::new(AtomicBool::new(false));
        let mut function_service = FunctionRunnerServiceServer::new(Gate {
            function,
            ready: is_ready.clone(),
        });
        if let Some(size) = args.max_recv_message_size {
            function_service = function_service.max_decoding_message_size(size);
        }

        let reflection_v1 = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(crate::proto::FILE_DESCRIPTOR_SET)
            .build_v1()?;
        let reflection_v1alpha = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(crate::proto::FILE_DESCRIPTOR_SET)
            .build_v1alpha()?;

        // The health statuses are set before the server listens, so no
        // probe sees SERVING ahead of the readiness future.
        let (health_reporter, health_service) = tonic_health::server::health_reporter();
        let (not_ready_tx, not_ready_rx) = tokio::sync::oneshot::channel::<BoxError>();
        match ready {
            None => {
                is_ready.store(true, Ordering::Release);
                set_health(&health_reporter, ServingStatus::Serving).await;
            }
            Some(ready) => {
                set_health(&health_reporter, ServingStatus::NotServing).await;
                // The future runs in a task of its own so that a panic in it
                // fails readiness instead of leaving the server NOT_SERVING.
                let loading = tokio::spawn(ready);
                tokio::spawn(async move {
                    match loading.await.unwrap_or_else(|e| Err(e.into())) {
                        Ok(()) => {
                            is_ready.store(true, Ordering::Release);
                            set_health(&health_reporter, ServingStatus::Serving).await;
                            tracing::info!("function is ready");
                        }
                        Err(e) => {
                            let _ = not_ready_tx.send(e);
                        }
                    }
                });
            }
        }

        // The metrics endpoint runs beside the gRPC server, like the Go SDK's;
        // a failure to serve it is logged, never the function's failure. The
        // counting layer below is always installed - with the endpoint disabled
        // the counts are simply never served, which is indistinguishable from
        // the Go SDK installing no interceptor.
        if !args.metrics_address.is_empty() {
            crate::metrics::initialize();
            let metrics_address = args.metrics_address.clone();
            let registry = crate::metrics::served_registry(metrics_registry);
            tokio::spawn(async move {
                if let Err(e) = crate::metrics::serve(&metrics_address, registry).await {
                    tracing::error!(error = %e, "cannot serve metrics");
                }
            });
        }

        tracing::info!(%address, insecure = args.insecure, "serving FunctionRunnerService");

        // A readiness failure shuts the server down like a signal does.
        // Without a readiness future, or once it succeeds, the receive never
        // yields Ok, and select! waits for the signal alone.
        let mut not_ready = None;
        builder
            .layer(crate::metrics::MetricsLayer)
            .add_service(function_service)
            .add_service(health_service)
            .add_service(reflection_v1)
            .add_service(reflection_v1alpha)
            .serve_with_shutdown(address, async {
                tokio::select! {
                    () = shutdown_signal() => {}
                    Ok(e) = not_ready_rx => not_ready = Some(e),
                }
            })
            .await?;

        match not_ready {
            Some(e) => Err(Error::NotReady(e)),
            None => Ok(()),
        }
    }
}

/// Sets the overall ("") and the FunctionRunnerService health status.
async fn set_health(reporter: &HealthReporter, status: ServingStatus) {
    for service in ["", SERVICE_NAME] {
        reporter.set_service_status(service, status).await;
    }
}

/// Answers RunFunction with UNAVAILABLE until the function is ready.
struct Gate<F> {
    function: F,
    ready: Arc<AtomicBool>,
}

#[tonic::async_trait]
impl<F: FunctionRunnerService> FunctionRunnerService for Gate<F> {
    async fn run_function(
        &self,
        request: Request<RunFunctionRequest>,
    ) -> Result<Response<RunFunctionResponse>, Status> {
        if !self.ready.load(Ordering::Acquire) {
            return Err(Status::unavailable("the function is not ready"));
        }
        self.function.run_function(request).await
    }
}

fn tls_config(dir: &Path) -> Result<ServerTlsConfig, Error> {
    let read = |name: &str| {
        let path = dir.join(name);
        std::fs::read(&path).map_err(|source| Error::ReadCertificate { path, source })
    };
    let cert = read("tls.crt")?;
    let key = read("tls.key")?;
    let ca = read("ca.crt")?;

    Ok(ServerTlsConfig::new()
        .identity(Identity::from_pem(cert, key))
        .client_ca_root(Certificate::from_pem(ca))
        .client_auth_optional(false))
}

#[cfg(unix)]
async fn shutdown_signal() {
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("cannot install SIGTERM handler");
    tokio::select! {
        _ = sigterm.recv() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
    tracing::info!("shutting down");
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}
