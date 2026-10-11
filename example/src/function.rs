//! An example composition function.
//!
//! Composes an S3 Bucket whose region comes from the XR's spec.region and
//! whose API version comes from the function's input, mirroring the
//! function-template-python example.

use function_sdk_rust::proto::v1::function_runner_service_server::FunctionRunnerService;
use function_sdk_rust::proto::v1::{RunFunctionRequest, RunFunctionResponse};
use function_sdk_rust::{request, resource, response};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct Function;

#[derive(serde::Deserialize)]
struct Xr {
    spec: XrSpec,
}

#[derive(serde::Deserialize)]
struct XrSpec {
    region: String,
}

#[tonic::async_trait]
impl FunctionRunnerService for Function {
    async fn run_function(
        &self,
        request: Request<RunFunctionRequest>,
    ) -> Result<Response<RunFunctionResponse>, Status> {
        let req = request.into_inner();
        let tag = req.meta.as_ref().map(|m| m.tag.clone()).unwrap_or_default();
        tracing::info!(tag, "running function");

        let mut rsp = response::to(&req, response::DEFAULT_TTL);

        let input = req
            .input
            .as_ref()
            .map(resource::struct_to_json)
            .unwrap_or_default();
        let Some(version) = input.get("version").and_then(|v| v.as_str()) else {
            response::fatal(&mut rsp, "invalid function input: version is required");
            return Ok(Response::new(rsp));
        };

        let Ok(observed_xr) = resource::get::<Xr>(request::observed_composite(&req)) else {
            response::fatal(&mut rsp, "invalid XR: spec.region is required");
            return Ok(Response::new(rsp));
        };
        let region = observed_xr.spec.region;

        resource::update(
            response::desired_composed(&mut rsp, "bucket"),
            &serde_json::json!({
                "apiVersion": format!("s3.aws.upbound.io/{version}"),
                "kind": "Bucket",
                "spec": {"forProvider": {"region": region}},
            }),
        )
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(rsp))
    }
}
