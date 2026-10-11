//! The `Server` builder's options. A separate test binary from serve.rs,
//! whose exact gRPC metric counts would see these tests' calls.

mod common;

use std::sync::{Arc, OnceLock};

use common::{connect_with_retry, free_port};
use function_sdk_rust::proto::v1::function_runner_service_server::FunctionRunnerService;
use function_sdk_rust::proto::v1::{RunFunctionRequest, RunFunctionResponse};
use function_sdk_rust::{Args, Error, Server, response};
use tonic::{Code, Request, Response, Status};
use tonic_health::pb::HealthCheckRequest;
use tonic_health::pb::health_check_response::ServingStatus;
use tonic_health::pb::health_client::HealthClient;

const SERVICES: [&str; 2] = ["", "apiextensions.fn.proto.v1.FunctionRunnerService"];

/// Answers with the bundle its readiness future loaded.
#[derive(Default)]
struct BundleFunction {
    bundle: Arc<OnceLock<String>>,
}

#[tonic::async_trait]
impl FunctionRunnerService for BundleFunction {
    async fn run_function(
        &self,
        request: Request<RunFunctionRequest>,
    ) -> Result<Response<RunFunctionResponse>, Status> {
        let req = request.into_inner();
        let mut rsp = response::to(&req, response::DEFAULT_TTL);
        let bundle = self
            .bundle
            .get()
            .expect("Server::ready gates on the bundle");
        response::normal(&mut rsp, bundle.clone());
        Ok(Response::new(rsp))
    }
}

fn insecure_args(port: u16) -> Args {
    Args {
        debug: false,
        address: format!("127.0.0.1:{port}"),
        tls_certs_dir: None,
        insecure: true,
        max_recv_message_size: None,
        metrics_address: String::new(),
    }
}

async fn health_status(health: &mut HealthClient<tonic::transport::Channel>, service: &str) -> i32 {
    health
        .check(HealthCheckRequest {
            service: service.to_string(),
        })
        .await
        .expect("health check must succeed")
        .into_inner()
        .status
}

#[tokio::test]
async fn ready_holds_the_function_back_until_it_completes() {
    let port = free_port();
    let (loaded_tx, loaded_rx) = tokio::sync::oneshot::channel::<()>();
    let function = BundleFunction::default();
    let bundle = function.bundle.clone();
    tokio::spawn(async move {
        let ready = async move {
            loaded_rx.await.expect("the test sends");
            bundle.set("v1".to_string()).expect("set once");
            Ok::<(), std::io::Error>(())
        };
        Server::new(function, &insecure_args(port))
            .ready(ready)
            .serve()
            .await
            .expect("serve must start");
    });

    let mut client = connect_with_retry(port).await;
    let channel = tonic::transport::Channel::from_shared(format!("http://127.0.0.1:{port}"))
        .expect("valid URI")
        .connect()
        .await
        .expect("health channel must connect");
    let mut health = HealthClient::new(channel);

    for service in SERVICES {
        assert_eq!(
            health_status(&mut health, service).await,
            ServingStatus::NotServing as i32,
            "{service:?} before ready"
        );
    }
    let err = client
        .run_function(RunFunctionRequest::default())
        .await
        .expect_err("RunFunction must wait for ready");
    assert_eq!(err.code(), Code::Unavailable);

    loaded_tx.send(()).expect("the readiness future waits");
    for service in SERVICES {
        let mut status = 0;
        for _ in 0..50 {
            status = health_status(&mut health, service).await;
            if status == ServingStatus::Serving as i32 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(
            status,
            ServingStatus::Serving as i32,
            "{service:?} after ready"
        );
    }
    let rsp = client
        .run_function(RunFunctionRequest::default())
        .await
        .expect("RunFunction must succeed once ready")
        .into_inner();
    assert_eq!(rsp.results[0].message, "v1");
}

#[tokio::test]
async fn a_failed_ready_stops_the_server() {
    let ready = async { Err::<(), _>(std::io::Error::other("cannot pull the bundle")) };
    let err = Server::new(BundleFunction::default(), &insecure_args(free_port()))
        .ready(ready)
        .serve()
        .await
        .expect_err("serve must return the readiness failure");
    assert!(matches!(err, Error::NotReady(_)), "{err:?}");
    assert_eq!(
        err.to_string(),
        "the function did not become ready: cannot pull the bundle"
    );
}

#[tokio::test]
async fn a_panicking_ready_stops_the_server() {
    let ready = async { panic!("cannot parse the bundle") };
    let err = Server::new(BundleFunction::default(), &insecure_args(free_port()))
        .ready::<_, std::io::Error>(ready)
        .serve()
        .await
        .expect_err("serve must return the readiness panic");
    assert!(matches!(err, Error::NotReady(_)), "{err:?}");
    assert!(
        err.to_string().contains("cannot parse the bundle"),
        "the panic message: {err}"
    );
}
