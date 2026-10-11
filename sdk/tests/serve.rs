mod common;

use common::{connect_with_retry, free_port, http_get};
use function_sdk_rust::proto::v1::function_runner_service_server::FunctionRunnerService;
use function_sdk_rust::proto::v1::{RequestMeta, RunFunctionRequest, RunFunctionResponse};
use function_sdk_rust::{Args, response, serve};
use tonic::{Request, Response, Status};

struct EchoFunction;

#[tonic::async_trait]
impl FunctionRunnerService for EchoFunction {
    async fn run_function(
        &self,
        request: Request<RunFunctionRequest>,
    ) -> Result<Response<RunFunctionResponse>, Status> {
        let req = request.into_inner();
        let mut rsp = response::to(&req, response::DEFAULT_TTL);
        response::normal(&mut rsp, "echo");
        Ok(Response::new(rsp))
    }
}

#[tokio::test]
async fn serve_handles_a_run_function_request() {
    let port = free_port();
    let metrics_port = free_port();
    tokio::spawn(async move {
        let args = Args {
            debug: false,
            address: format!("127.0.0.1:{port}"),
            tls_certs_dir: None,
            insecure: true,
            max_recv_message_size: None,
            metrics_address: format!("127.0.0.1:{metrics_port}"),
        };
        serve(EchoFunction, &args).await.expect("serve must start");
    });

    let mut client = connect_with_retry(port).await;

    let rsp = client
        .run_function(RunFunctionRequest {
            meta: Some(RequestMeta {
                tag: "smoke".to_string(),
                capabilities: vec![],
            }),
            ..Default::default()
        })
        .await
        .expect("RunFunction must succeed")
        .into_inner();

    assert_eq!(rsp.meta.unwrap().tag, "smoke");
    assert_eq!(rsp.results[0].message, "echo");

    let channel = tonic::transport::Channel::from_shared(format!("http://127.0.0.1:{port}"))
        .expect("valid URI")
        .connect()
        .await
        .expect("health channel must connect");
    let mut health = tonic_health::pb::health_client::HealthClient::new(channel);
    let status = health
        .check(tonic_health::pb::HealthCheckRequest {
            service: "apiextensions.fn.proto.v1.FunctionRunnerService".to_string(),
        })
        .await
        .expect("health check must succeed")
        .into_inner();
    assert_eq!(
        status.status,
        tonic_health::pb::health_check_response::ServingStatus::Serving as i32
    );

    // The Go SDK's gRPC server series, served as OpenMetrics: one
    // RunFunction started, received, handled OK and answered. The handled
    // count lands when the response's trailers go out, which can trail the
    // client seeing the response, so it is polled briefly.
    const RUN_FUNCTION: &str = "grpc_type=\"unary\",grpc_service=\"apiextensions.fn.proto.v1.FunctionRunnerService\",grpc_method=\"RunFunction\"";
    let mut om = String::new();
    for _ in 0..50 {
        om = http_get(metrics_port, None).await;
        if om.contains(&format!(
            "grpc_server_handled_total{{{RUN_FUNCTION},grpc_code=\"OK\"}} 1"
        )) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(
        om.contains("application/openmetrics-text; version=1.0.0; charset=utf-8"),
        "OpenMetrics is the main format: {om}"
    );
    assert!(om.trim_end().ends_with("# EOF"), "OpenMetrics body: {om}");
    assert!(
        om.contains(&format!("grpc_server_started_total{{{RUN_FUNCTION}}} 1")),
        "started: {om}"
    );
    assert!(
        om.contains(&format!(
            "grpc_server_msg_received_total{{{RUN_FUNCTION}}} 1"
        )),
        "msg_received: {om}"
    );
    assert!(
        om.contains(&format!(
            "grpc_server_handled_total{{{RUN_FUNCTION},grpc_code=\"OK\"}} 1"
        )),
        "handled OK: {om}"
    );
    assert!(
        om.contains(&format!("grpc_server_msg_sent_total{{{RUN_FUNCTION}}} 1")),
        "msg_sent: {om}"
    );
    // The health Check above was counted too - the layer covers the whole
    // router, like the Go SDK's server-wide interceptor.
    assert!(
        om.contains("grpc_server_started_total{grpc_type=\"unary\",grpc_service=\"grpc.health.v1.Health\",grpc_method=\"Check\"} 1"),
        "health Check counted: {om}"
    );
    // InitializeMetrics parity: streaming methods exist as zero series and
    // are never incremented (the Go interceptor was unary-only too).
    assert!(
        om.contains("grpc_server_started_total{grpc_type=\"server_stream\",grpc_service=\"grpc.health.v1.Health\",grpc_method=\"Watch\"} 0"),
        "streaming methods stay zero: {om}"
    );

    // A scraper that asks for the classic Prometheus text format without
    // accepting OpenMetrics gets it.
    let classic = http_get(metrics_port, Some("text/plain; version=0.0.4")).await;
    assert!(
        classic.contains("text/plain; version=0.0.4"),
        "classic on request: {classic}"
    );
    assert!(classic.contains("# TYPE grpc_server_started_total counter"));
    assert!(!classic.contains("# EOF"));
}
