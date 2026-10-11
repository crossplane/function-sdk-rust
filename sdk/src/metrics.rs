//! The gRPC server metrics function-sdk-go serves, re-created for this
//! SDK's server: `grpc_server_started_total`, `grpc_server_handled_total`,
//! `grpc_server_msg_received_total` and `grpc_server_msg_sent_total`, with
//! the names, labels and help strings of the grpc-prometheus interceptor
//! the Go SDK installs - so dashboards and alerts work across Go and Rust
//! functions unchanged. Like that interceptor, which was unary-only,
//! streaming methods (reflection, health Watch) are not counted; like its
//! InitializeMetrics, every method the server carries starts as a zero
//! series; and like the Go SDK, there is no handling-time histogram (it
//! never enabled one).
//!
//! The /metrics endpoint (`--metrics-address`, the Go SDK's `:8080`
//! default, empty disables) serves OpenMetrics 1.0 as its main format,
//! straight from prometheus-client's encoder; a scraper whose Accept
//! header asks for the classic Prometheus text format without accepting
//! OpenMetrics gets that instead, derived from the same rendering.

use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use std::task::{Context, Poll};

use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const FUNCTION_SERVICE: &str = "apiextensions.fn.proto.v1.FunctionRunnerService";
const HEALTH_SERVICE: &str = "grpc.health.v1.Health";

type Labels = Vec<(String, String)>;

/// A counter family addressed by label values in a fixed key order. The
/// name registers without its `_total` suffix and the help without its
/// trailing period - the OpenMetrics encoder appends both - so the served
/// text is byte-identical to the Go SDK's.
struct LabeledCounter {
    name: &'static str,
    help: &'static str,
    keys: &'static [&'static str],
    family: Family<Labels, Counter>,
}

impl LabeledCounter {
    fn new(name: &'static str, help: &'static str, keys: &'static [&'static str]) -> Self {
        LabeledCounter {
            name,
            help,
            keys,
            family: Family::default(),
        }
    }

    fn register(&self, registry: &mut Registry) {
        let help = self.help.strip_suffix('.').unwrap_or(self.help);
        registry.register(self.name, help, self.family.clone());
    }

    fn with_label_values(&self, values: &[&str]) -> impl std::ops::Deref<Target = Counter> {
        debug_assert_eq!(self.keys.len(), values.len());
        let labels: Labels = self
            .keys
            .iter()
            .zip(values)
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.family.get_or_create(&labels)
    }
}

static STARTED: LazyLock<LabeledCounter> = LazyLock::new(|| {
    LabeledCounter::new(
        "grpc_server_started",
        "Total number of RPCs started on the server.",
        &["grpc_type", "grpc_service", "grpc_method"],
    )
});

static HANDLED: LazyLock<LabeledCounter> = LazyLock::new(|| {
    LabeledCounter::new(
        "grpc_server_handled",
        "Total number of RPCs completed on the server, regardless of success or failure.",
        &["grpc_type", "grpc_service", "grpc_method", "grpc_code"],
    )
});

static MSG_RECEIVED: LazyLock<LabeledCounter> = LazyLock::new(|| {
    LabeledCounter::new(
        "grpc_server_msg_received",
        "Total number of RPC stream messages received on the server.",
        &["grpc_type", "grpc_service", "grpc_method"],
    )
});

static MSG_SENT: LazyLock<LabeledCounter> = LazyLock::new(|| {
    LabeledCounter::new(
        "grpc_server_msg_sent",
        "Total number of gRPC stream messages sent by the server.",
        &["grpc_type", "grpc_service", "grpc_method"],
    )
});

/// The gRPC status code names, indexed by their wire value - the strings
/// google.golang.org/grpc/codes renders, which is what the Go SDK's
/// grpc_code label carried.
const CODES: [&str; 17] = [
    "OK",
    "Canceled",
    "Unknown",
    "InvalidArgument",
    "DeadlineExceeded",
    "NotFound",
    "AlreadyExists",
    "PermissionDenied",
    "ResourceExhausted",
    "FailedPrecondition",
    "Aborted",
    "OutOfRange",
    "Unimplemented",
    "Internal",
    "Unavailable",
    "DataLoss",
    "Unauthenticated",
];

fn code_str(n: u32) -> &'static str {
    CODES.get(n as usize).copied().unwrap_or("Unknown")
}

/// The unary methods the server carries - the only calls the Go SDK's
/// interceptor counted (it installed no stream interceptor).
fn unary_method(path: &str) -> Option<(&'static str, &'static str)> {
    match path {
        "/apiextensions.fn.proto.v1.FunctionRunnerService/RunFunction" => {
            Some((FUNCTION_SERVICE, "RunFunction"))
        }
        "/grpc.health.v1.Health/Check" => Some((HEALTH_SERVICE, "Check")),
        _ => None,
    }
}

/// The registry the /metrics endpoint serves: the function's own, when it
/// set one with [`crate::Server::metrics_registry`], else an empty one, with
/// the gRPC server series added at its root. The series stay process-wide
/// statics so the counting layer needs no registry.
pub(crate) fn served_registry(registry: Option<Registry>) -> Registry {
    let mut registry = registry.unwrap_or_default();
    for counter in [&*STARTED, &*HANDLED, &*MSG_RECEIVED, &*MSG_SENT] {
        counter.register(&mut registry);
    }
    registry
}

/// Pre-creates every series the server can emit at zero, the way
/// grpc-prometheus's InitializeMetrics did for the Go SDK: a scrape sees
/// each method of each registered service from the first request on,
/// streaming methods included (present and forever zero, exactly as under
/// the Go SDK's unary-only interceptor).
pub(crate) fn initialize() {
    let methods: [(&str, &str, &str); 5] = [
        ("unary", FUNCTION_SERVICE, "RunFunction"),
        ("unary", HEALTH_SERVICE, "Check"),
        ("server_stream", HEALTH_SERVICE, "Watch"),
        (
            "bidi_stream",
            "grpc.reflection.v1.ServerReflection",
            "ServerReflectionInfo",
        ),
        (
            "bidi_stream",
            "grpc.reflection.v1alpha.ServerReflection",
            "ServerReflectionInfo",
        ),
    ];
    for (kind, service, method) in methods {
        let labels = [kind, service, method];
        let _ = STARTED.with_label_values(&labels);
        let _ = MSG_RECEIVED.with_label_values(&labels);
        let _ = MSG_SENT.with_label_values(&labels);
        for code in CODES {
            let _ = HANDLED.with_label_values(&[kind, service, method, code]);
        }
    }
}

/// One unary call's bookkeeping: started and msg_received on creation,
/// handled (and msg_sent on OK) exactly once when the status is known. A
/// reporter dropped before any status - the caller went away mid-call - is
/// a Canceled, which is what the Go interceptor reported for a canceled
/// context.
struct Reporter {
    service: &'static str,
    method: &'static str,
    done: bool,
}

impl Reporter {
    fn start(service: &'static str, method: &'static str) -> Self {
        STARTED.with_label_values(&["unary", service, method]).inc();
        MSG_RECEIVED
            .with_label_values(&["unary", service, method])
            .inc();
        Reporter {
            service,
            method,
            done: false,
        }
    }

    fn report(&mut self, code: u32) {
        if self.done {
            return;
        }
        self.done = true;
        HANDLED
            .with_label_values(&["unary", self.service, self.method, code_str(code)])
            .inc();
        if code == 0 {
            MSG_SENT
                .with_label_values(&["unary", self.service, self.method])
                .inc();
        }
    }
}

impl Drop for Reporter {
    fn drop(&mut self) {
        if !self.done {
            self.done = true;
            HANDLED
                .with_label_values(&["unary", self.service, self.method, "Canceled"])
                .inc();
        }
    }
}

/// Reads a numeric grpc-status from headers or trailers; a gRPC response
/// with no grpc-status is out of spec, so the caller falls back to Unknown.
fn status_in(map: &http::HeaderMap) -> Option<u32> {
    map.get("grpc-status")?.to_str().ok()?.parse().ok()
}

/// The tower layer serve() wraps the whole router in, so the health service
/// is counted exactly as the Go SDK's server-wide interceptor counted it.
#[derive(Clone, Copy, Default)]
pub(crate) struct MetricsLayer;

impl<S> tower_layer::Layer<S> for MetricsLayer {
    type Service = Metrics<S>;

    fn layer(&self, inner: S) -> Metrics<S> {
        Metrics { inner }
    }
}

#[derive(Clone)]
pub(crate) struct Metrics<S> {
    inner: S,
}

impl<S, ReqBody> tower_service::Service<http::Request<ReqBody>> for Metrics<S>
where
    S: tower_service::Service<http::Request<ReqBody>, Response = http::Response<tonic::body::Body>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = http::Response<MetricsBody>;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let unary = unary_method(req.uri().path());
        // The readied service is the one that must take the call; the clone
        // parks in self (the tower Service contract).
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            let mut reporter = unary.map(|(service, method)| Reporter::start(service, method));
            let rsp = inner.call(req).await?;
            let (parts, body) = rsp.into_parts();
            // A trailers-only response (tonic's immediate refusals) carries
            // grpc-status in the headers; report now and wrap nothing.
            if let (Some(r), Some(code)) = (reporter.as_mut(), status_in(&parts.headers)) {
                r.report(code);
                reporter = None;
            }
            Ok(http::Response::from_parts(
                parts,
                MetricsBody {
                    inner: body,
                    reporter,
                },
            ))
        })
    }
}

/// The response body with the call's reporter riding along: the status of a
/// normal unary response is in the trailers frame, so that is where handled
/// is counted - and a body dropped before its trailers is the client going
/// away, the reporter's Canceled.
pub(crate) struct MetricsBody {
    inner: tonic::body::Body,
    reporter: Option<Reporter>,
}

impl http_body::Body for MetricsBody {
    type Data = bytes::Bytes;
    type Error = tonic::Status;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();
        match std::task::ready!(Pin::new(&mut this.inner).poll_frame(cx)) {
            Some(Ok(frame)) => {
                if let (Some(trailers), Some(r)) = (frame.trailers_ref(), this.reporter.as_mut()) {
                    r.report(status_in(trailers).unwrap_or(2));
                }
                Poll::Ready(Some(Ok(frame)))
            }
            Some(Err(e)) => {
                if let Some(r) = this.reporter.as_mut() {
                    r.report(2);
                }
                Poll::Ready(Some(Err(e)))
            }
            None => {
                // A stream that ends without trailers never carried a
                // status; out of spec, counted as Unknown like a broken
                // response would be.
                if let Some(r) = this.reporter.as_mut() {
                    r.report(2);
                }
                Poll::Ready(None)
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        http_body::Body::size_hint(&self.inner)
    }
}

const OPENMETRICS_CONTENT_TYPE: &str = "application/openmetrics-text; version=1.0.0; charset=utf-8";

/// The registry rendered as OpenMetrics 1.0 text, straight from
/// prometheus-client's encoder.
fn render_openmetrics(registry: &Registry) -> String {
    let mut out = String::new();
    let _ = prometheus_client::encoding::text::encode(&mut out, registry);
    out
}

/// The classic Prometheus text format (`text/plain; version=0.0.4`),
/// derived from the OpenMetrics rendering: the two differ only in a counter
/// family being named with its `_total` suffix in the classic HELP and TYPE
/// lines, and in the terminating `# EOF` - the samples are identical.
fn classic_from_openmetrics(om: &str) -> String {
    use std::fmt::Write as _;
    let counters: std::collections::HashSet<&str> = om
        .lines()
        .filter_map(|l| l.strip_prefix("# TYPE ")?.strip_suffix(" counter"))
        .collect();
    let mut out = String::new();
    for line in om.lines() {
        if line == "# EOF" {
            continue;
        }
        if let Some(name) = line
            .strip_prefix("# TYPE ")
            .and_then(|rest| rest.strip_suffix(" counter"))
        {
            let _ = writeln!(out, "# TYPE {name}_total counter");
            continue;
        }
        if let Some((name, help)) = line
            .strip_prefix("# HELP ")
            .and_then(|rest| rest.split_once(' '))
            && counters.contains(name)
        {
            let _ = writeln!(out, "# HELP {name}_total {help}");
            continue;
        }
        let _ = writeln!(out, "{line}");
    }
    out
}

/// Whether the request's Accept header asks for the classic Prometheus text
/// format without accepting OpenMetrics. OpenMetrics is the main format: it
/// is served when the header names it (whatever the q-values), when there
/// is no Accept header, and for catch-alls like */*; only an explicit
/// text/plain-and-not-OpenMetrics preference gets the classic rendering.
fn wants_classic_text(request_head: &str) -> bool {
    let Some(accept) = request_head.lines().find_map(|l| {
        let (name, value) = l.split_once(':')?;
        name.trim().eq_ignore_ascii_case("accept").then_some(value)
    }) else {
        return false;
    };
    !accept.contains("application/openmetrics-text") && accept.contains("text/plain")
}

/// Serves the registry at /metrics on address until the process ends - the
/// plain-HTTP listener the Go SDK starts beside its gRPC server.
pub(crate) async fn serve(address: &str, registry: Registry) -> Result<(), std::io::Error> {
    let registry = Arc::new(registry);
    // The Go SDK's ":8080" shorthand means every interface.
    let address = if address.starts_with(':') {
        format!("0.0.0.0{address}")
    } else {
        address.to_string()
    };
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!(address = %address, "serving metrics");
    loop {
        let Ok((mut conn, _)) = listener.accept().await else {
            continue;
        };
        let registry = registry.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            let n = conn.read(&mut buf).await.unwrap_or(0);
            let head = String::from_utf8_lossy(&buf[..n]).into_owned();
            let path = head.split_whitespace().nth(1).unwrap_or_default();
            let response = if path == "/metrics" {
                let om = render_openmetrics(&registry);
                let (body, content_type) = if wants_classic_text(&head) {
                    (classic_from_openmetrics(&om), "text/plain; version=0.0.4")
                } else {
                    (om, OPENMETRICS_CONTENT_TYPE)
                };
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
            } else {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_string()
            };
            let _ = conn.write_all(response.as_bytes()).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_names_match_grpc_go() {
        assert_eq!(code_str(0), "OK");
        assert_eq!(code_str(1), "Canceled");
        assert_eq!(code_str(12), "Unimplemented");
        assert_eq!(code_str(16), "Unauthenticated");
        assert_eq!(code_str(17), "Unknown");
    }

    #[test]
    fn only_unary_methods_are_counted() {
        assert_eq!(
            unary_method("/apiextensions.fn.proto.v1.FunctionRunnerService/RunFunction"),
            Some((FUNCTION_SERVICE, "RunFunction"))
        );
        assert_eq!(
            unary_method("/grpc.health.v1.Health/Check"),
            Some((HEALTH_SERVICE, "Check"))
        );
        // Streaming methods stay uncounted, as under the Go SDK's
        // unary-only interceptor.
        assert_eq!(unary_method("/grpc.health.v1.Health/Watch"), None);
        assert_eq!(
            unary_method("/grpc.reflection.v1.ServerReflection/ServerReflectionInfo"),
            None
        );
    }

    #[test]
    fn openmetrics_is_the_main_format() {
        assert!(!wants_classic_text("GET /metrics HTTP/1.1\r\nHost: x\r\n"));
        assert!(!wants_classic_text(
            "GET /metrics HTTP/1.1\r\nAccept: */*\r\n"
        ));
        assert!(!wants_classic_text(
            "GET /metrics HTTP/1.1\r\nAccept: application/openmetrics-text;version=1.0.0;q=0.9,text/plain;version=0.0.4;q=0.5\r\n"
        ));
        assert!(wants_classic_text(
            "GET /metrics HTTP/1.1\r\nAccept: text/plain; version=0.0.4\r\n"
        ));
    }

    #[test]
    fn renderings_differ_only_where_the_formats_do() {
        initialize();
        let om = render_openmetrics(&served_registry(None));
        assert!(om.ends_with("# EOF\n"), "OpenMetrics ends with EOF:\n{om}");
        assert!(
            om.contains("# TYPE grpc_server_started counter"),
            "counter family named without _total:\n{om}"
        );
        assert!(
            om.contains("# HELP grpc_server_started Total number of RPCs started on the server."),
            "the Go SDK's help string, single trailing period:\n{om}"
        );
        assert!(!om.contains("server.."), "no doubled period:\n{om}");

        let classic = classic_from_openmetrics(&om);
        assert!(classic.contains("# TYPE grpc_server_started_total counter"));
        assert!(classic.contains("# HELP grpc_server_started_total "));
        assert!(!classic.contains("# EOF"));
    }

    #[test]
    fn a_dropped_reporter_is_a_canceled_call() {
        initialize();
        let registry = served_registry(None);
        let canceled = || {
            render_openmetrics(&registry)
                .lines()
                .find(|l| {
                    l.starts_with("grpc_server_handled_total{grpc_type=\"unary\"")
                        && l.contains("grpc_method=\"RunFunction\"")
                        && l.contains("grpc_code=\"Canceled\"")
                })
                .and_then(|l| l.rsplit(' ').next()?.parse::<f64>().ok())
                .unwrap_or(0.0)
        };
        let before = canceled();
        drop(Reporter::start(FUNCTION_SERVICE, "RunFunction"));
        assert_eq!(canceled(), before + 1.0);
        // A reported call does not double-count on drop.
        let mut r = Reporter::start(FUNCTION_SERVICE, "RunFunction");
        r.report(0);
        drop(r);
        assert_eq!(canceled(), before + 1.0);
    }
}
