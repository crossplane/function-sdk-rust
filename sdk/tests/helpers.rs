use std::time::Duration;

use function_sdk_rust::proto::v1::{
    Capability, Ready, RunFunctionRequest, RunFunctionResponse, Severity, Target,
};
use function_sdk_rust::{Error, context, request, resource, response};

fn fixture() -> RunFunctionRequest {
    serde_json::from_str(
        r#"{
            "meta": {"tag": "test-tag", "capabilities": ["CAPABILITY_CAPABILITIES", "CAPABILITY_CONDITIONS"]},
            "observed": {
                "composite": {
                    "resource": {
                        "apiVersion": "example.org/v1",
                        "kind": "XR",
                        "metadata": {"name": "my-xr"},
                        "spec": {"region": "eu-central-1"},
                        "status": {
                            "conditions": [
                                {"type": "Ready", "status": "True", "reason": "Available"}
                            ]
                        }
                    }
                }
            },
            "desired": {
                "resources": {
                    "bucket": {"resource": {"apiVersion": "s3.aws.upbound.io/v1beta2", "kind": "Bucket"}}
                }
            },
            "input": {"version": "v1beta2"},
            "context": {"apiextensions.crossplane.io/environment": {"key": "value"}}
        }"#,
    )
    .expect("fixture must be valid protojson")
}

#[test]
fn request_parses_from_protojson() {
    let req = fixture();
    assert_eq!(req.meta.as_ref().unwrap().tag, "test-tag");
    assert!(request::advertises_capabilities(&req));
    assert!(request::has_capability(&req, Capability::Conditions));
    assert!(!request::has_capability(&req, Capability::RequiredSchemas));
}

#[test]
fn response_to_copies_tag_desired_and_context() {
    let req = fixture();
    let rsp = response::to(&req, response::DEFAULT_TTL);

    assert_eq!(rsp.meta.as_ref().unwrap().tag, "test-tag");
    assert_eq!(rsp.meta.as_ref().unwrap().ttl.as_ref().unwrap().seconds, 60);
    assert_eq!(rsp.desired, req.desired);
    assert_eq!(rsp.context, req.context);
}

#[test]
fn results_carry_severity_and_default_target() {
    let mut rsp = RunFunctionResponse::default();
    response::normal(&mut rsp, "all good").reason = Some("AllGood".to_string());
    let warning = response::warning(&mut rsp, "hmm");
    warning.target = Some(Target::CompositeAndClaim as i32);
    response::fatal(&mut rsp, "boom");

    assert_eq!(rsp.results.len(), 3);
    assert_eq!(rsp.results[0].severity, Severity::Normal as i32);
    assert_eq!(rsp.results[0].reason.as_deref(), Some("AllGood"));
    assert_eq!(rsp.results[0].target, Some(Target::Composite as i32));
    assert_eq!(
        rsp.results[1].target,
        Some(Target::CompositeAndClaim as i32)
    );
    assert_eq!(rsp.results[2].severity, Severity::Fatal as i32);
    assert_eq!(rsp.results[2].message, "boom");
}

#[test]
fn get_input_deserializes_typed_input() {
    #[derive(serde::Deserialize)]
    struct Input {
        version: String,
    }

    let input: Input = request::get_input(&fixture()).unwrap();
    assert_eq!(input.version, "v1beta2");

    let missing = request::get_input::<Input>(&RunFunctionRequest::default());
    assert!(matches!(missing, Err(Error::MissingInput)));
}

#[test]
fn resource_get_deserializes_the_composite_resource() {
    #[derive(serde::Deserialize)]
    struct Spec {
        region: String,
    }
    #[derive(serde::Deserialize)]
    struct Xr {
        spec: Spec,
    }

    let req = fixture();
    let composite = req.observed.as_ref().unwrap().composite.as_ref();
    let xr: Xr = resource::get(composite).unwrap();
    assert_eq!(xr.spec.region, "eu-central-1");

    let missing = resource::get::<Xr>(None);
    assert!(matches!(missing, Err(Error::MissingResource)));
}

#[test]
fn resource_get_deserializes_composed_resources() {
    #[derive(serde::Deserialize)]
    struct ForProvider {
        region: String,
    }
    #[derive(serde::Deserialize)]
    struct Spec {
        #[serde(rename = "forProvider")]
        for_provider: ForProvider,
    }
    #[derive(serde::Deserialize)]
    struct Bucket {
        spec: Spec,
    }

    let req: RunFunctionRequest = serde_json::from_str(
        r#"{"observed": {"resources": {"bucket": {"resource": {
            "apiVersion": "s3.aws.upbound.io/v1beta2",
            "kind": "Bucket",
            "spec": {"forProvider": {"region": "eu-central-1"}}
        }}}}}"#,
    )
    .unwrap();
    let resources = &req.observed.as_ref().unwrap().resources;

    let bucket: Bucket = resource::get(resources.get("bucket")).unwrap();
    assert_eq!(bucket.spec.for_provider.region, "eu-central-1");

    // A composed resource Crossplane hasn't observed yet is absent from the map.
    let missing = resource::get::<Bucket>(resources.get("nope"));
    assert!(matches!(missing, Err(Error::MissingResource)));
}

#[test]
fn context_keys_read_and_write() {
    let req = fixture();
    let environment = request::get_context_key(&req, context::KEY_ENVIRONMENT).unwrap();
    assert_eq!(environment["key"], "value");
    assert_eq!(request::get_context_key(&req, "nope"), None);

    let mut rsp = response::to(&req, response::DEFAULT_TTL);
    response::set_context_key(&mut rsp, "example.org/mine", &serde_json::json!({"a": 1})).unwrap();

    // The context copied forward from the request is preserved.
    let fields = &rsp.context.as_ref().unwrap().fields;
    assert!(fields.contains_key(context::KEY_ENVIRONMENT));
    assert!(fields.contains_key("example.org/mine"));
}

#[test]
fn set_ready_and_connection_details() {
    let mut r = function_sdk_rust::proto::v1::Resource::default();
    resource::set_ready(&mut r, Ready::True);
    resource::add_connection_detail(&mut r, "endpoint", "https://example.org");

    assert_eq!(r.ready, Ready::True as i32);
    assert_eq!(
        r.connection_details["endpoint"],
        b"https://example.org".to_vec()
    );
}

#[test]
fn update_overwrites_and_adds_top_level_fields() {
    let req = fixture();
    let mut rsp = response::to(&req, response::DEFAULT_TTL);
    let bucket = rsp
        .desired
        .as_mut()
        .unwrap()
        .resources
        .get_mut("bucket")
        .unwrap();

    resource::update(
        bucket,
        &serde_json::json!({
            "kind": "Bucket",
            "spec": {"forProvider": {"region": "eu-central-1"}},
        }),
    )
    .unwrap();

    let got = resource::struct_to_json(bucket.resource.as_ref().unwrap());
    assert_eq!(got["apiVersion"], "s3.aws.upbound.io/v1beta2");
    assert_eq!(got["spec"]["forProvider"]["region"], "eu-central-1");
}

#[test]
fn update_rejects_non_objects() {
    let mut r = function_sdk_rust::proto::v1::Resource::default();
    assert!(resource::update(&mut r, &serde_json::json!(42)).is_err());
}

#[test]
fn get_condition_reads_observed_conditions() {
    let req = fixture();
    let composite = req.observed.as_ref().unwrap().composite.as_ref();

    let ready = resource::get_condition(composite, "Ready");
    assert_eq!(ready.status, "True");
    assert_eq!(ready.reason.as_deref(), Some("Available"));

    let synced = resource::get_condition(composite, "Synced");
    assert_eq!(synced.status, "Unknown");

    let missing = resource::get_condition(None, "Ready");
    assert_eq!(missing.status, "Unknown");
}

#[test]
fn required_resources_distinguish_unresolved_from_empty() {
    let req = fixture();
    assert_eq!(request::get_required_resources(&req, "nope"), None);

    let req: RunFunctionRequest =
        serde_json::from_str(r#"{"requiredResources": {"empty": {}}}"#).unwrap();
    assert_eq!(request::get_required_resources(&req, "empty"), Some(vec![]));
}

#[test]
fn typed_required_resources_deserialize_items() {
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Metadata {
        name: String,
    }
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Spec {
        replicas: i64,
    }
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Provider {
        metadata: Metadata,
        spec: Spec,
    }

    // Integers arrive as f64 through structpb, like observed resources.
    let req: RunFunctionRequest = serde_json::from_str(
        r#"{"requiredResources": {
            "providers": {"items": [
                {"resource": {"metadata": {"name": "aws"}, "spec": {"replicas": 3.0}}},
                {"resource": {"metadata": {"name": "gcp"}, "spec": {"replicas": 1.0}}}
            ]}
        }}"#,
    )
    .unwrap();

    let providers: Vec<Provider> = request::get_required_resources_as(&req, "providers")
        .unwrap()
        .unwrap();
    assert_eq!(
        providers,
        vec![
            Provider {
                metadata: Metadata {
                    name: "aws".to_string()
                },
                spec: Spec { replicas: 3 },
            },
            Provider {
                metadata: Metadata {
                    name: "gcp".to_string()
                },
                spec: Spec { replicas: 1 },
            },
        ]
    );
    let first: Provider = request::get_required_resource_as(&req, "providers")
        .unwrap()
        .unwrap();
    assert_eq!(first.metadata.name, "aws");

    let untyped = request::get_required_resources_as::<serde_json::Value>(&req, "providers");
    assert_eq!(
        untyped.unwrap(),
        request::get_required_resources(&req, "providers")
    );
}

#[test]
fn typed_required_resources_distinguish_unresolved_from_empty() {
    let req: RunFunctionRequest =
        serde_json::from_str(r#"{"requiredResources": {"empty": {}}}"#).unwrap();

    let unresolved = request::get_required_resources_as::<serde_json::Value>(&req, "nope");
    assert_eq!(unresolved.unwrap(), None);
    let empty = request::get_required_resources_as::<serde_json::Value>(&req, "empty");
    assert_eq!(empty.unwrap(), Some(vec![]));

    let unresolved = request::get_required_resource_as::<serde_json::Value>(&req, "nope");
    assert_eq!(unresolved.unwrap(), None);
    let empty = request::get_required_resource_as::<serde_json::Value>(&req, "empty");
    assert_eq!(empty.unwrap(), None);
}

#[test]
fn typed_required_resources_report_mismatched_items() {
    #[derive(serde::Deserialize, Debug)]
    #[expect(dead_code, reason = "the item never deserializes")]
    struct Item {
        replicas: i64,
    }

    let req: RunFunctionRequest = serde_json::from_str(
        r#"{"requiredResources": {"providers": {"items": [
            {"resource": {"replicas": "three"}}
        ]}}}"#,
    )
    .unwrap();

    let all = request::get_required_resources_as::<Item>(&req, "providers");
    assert!(matches!(all, Err(Error::Json(_))));
    let one = request::get_required_resource_as::<Item>(&req, "providers");
    assert!(matches!(one, Err(Error::Json(_))));
}

#[test]
fn observed_accessors_borrow_the_observed_state() {
    let req: RunFunctionRequest = serde_json::from_str(
        r#"{"observed": {
            "composite": {"resource": {"kind": "XR"}},
            "resources": {"bucket": {"resource": {"kind": "Bucket"}}}
        }}"#,
    )
    .unwrap();
    let observed = req.observed.as_ref().unwrap();

    assert_eq!(
        request::observed_composite(&req),
        observed.composite.as_ref()
    );
    assert_eq!(
        request::observed_composed(&req, "bucket"),
        observed.resources.get("bucket")
    );
    assert_eq!(request::observed_composed(&req, "nope"), None);

    let empty = RunFunctionRequest::default();
    assert_eq!(request::observed_composite(&empty), None);
    assert_eq!(request::observed_composed(&empty, "bucket"), None);
}

#[test]
fn desired_accessors_insert_missing_resources() {
    let mut rsp = RunFunctionResponse::default();

    resource::update_status(
        response::desired_composite(&mut rsp),
        &serde_json::json!({"bucket": "ready"}),
    )
    .unwrap();
    resource::set_ready(response::desired_composite(&mut rsp), Ready::True);
    resource::update(
        response::desired_composed(&mut rsp, "bucket"),
        &serde_json::json!({"kind": "Bucket"}),
    )
    .unwrap();

    let desired = rsp.desired.unwrap();
    let xr = desired.composite.as_ref().unwrap();
    assert_eq!(xr.ready, Ready::True as i32);
    let xr: serde_json::Value = resource::get(Some(xr)).unwrap();
    assert_eq!(xr, serde_json::json!({"status": {"bucket": "ready"}}));
    let bucket: serde_json::Value = resource::get(desired.resources.get("bucket")).unwrap();
    assert_eq!(bucket, serde_json::json!({"kind": "Bucket"}));
}

#[test]
fn desired_accessors_keep_the_copied_forward_state() {
    let mut rsp = response::to(&fixture(), response::DEFAULT_TTL);

    let bucket = response::desired_composed(&mut rsp, "bucket");
    resource::set_ready(bucket, Ready::True);
    resource::update(
        bucket,
        &serde_json::json!({"spec": {"forProvider": {"region": "eu-central-1"}}}),
    )
    .unwrap();

    let desired = rsp.desired.unwrap();
    assert_eq!(desired.resources.len(), 1);
    let bucket = &desired.resources["bucket"];
    assert_eq!(bucket.ready, Ready::True as i32);
    let bucket: serde_json::Value = resource::get(Some(bucket)).unwrap();
    assert_eq!(
        bucket,
        serde_json::json!({
            "apiVersion": "s3.aws.upbound.io/v1beta2",
            "kind": "Bucket",
            "spec": {"forProvider": {"region": "eu-central-1"}},
        })
    );
}

#[test]
fn require_resources_builds_selectors() {
    let mut rsp = RunFunctionResponse::default();
    response::require_resources(
        &mut rsp,
        "vpcs",
        "ec2.aws.upbound.io/v1beta1",
        "VPC",
        Some(response::ResourceMatch::Name("my-vpc".to_string())),
        None,
    );

    let selector = &rsp.requirements.as_ref().unwrap().resources["vpcs"];
    assert_eq!(selector.kind, "VPC");
}

#[test]
fn conditions_can_set_message_and_target() {
    let mut rsp = RunFunctionResponse::default();
    let c = response::condition_true(&mut rsp, "DatabaseReady", "Available");
    c.message = Some("all databases up".to_string());

    assert_eq!(rsp.conditions.len(), 1);
    assert_eq!(rsp.conditions[0].r#type, "DatabaseReady");
    assert_eq!(
        rsp.conditions[0].message.as_deref(),
        Some("all databases up")
    );
}

#[test]
fn child_name_is_deterministic_and_dns_safe() {
    let a = resource::child_name(&["my-xr", "bucket"], "-");
    let b = resource::child_name(&["my-xr", "bucket"], "-");
    assert_eq!(a, b);
    assert!(a.starts_with("my-xr-bucket-"));
    assert_eq!(a.len(), "my-xr-bucket-".len() + 5);

    let long = "a".repeat(100);
    let name = resource::child_name(&[&long, "suffix"], "-");
    assert!(name.len() <= 63);
}

#[test]
fn struct_json_round_trip() {
    let json = serde_json::json!({
        "string": "value",
        "float": 1.5,
        "int": 3,
        "bool": true,
        "null": null,
        "list": [1, "two", {"three": 3}],
        "nested": {"deep": {"deeper": "value"}},
    });
    let serde_json::Value::Object(map) = &json else {
        unreachable!()
    };
    let s = resource::json_to_struct(map);
    assert_eq!(resource::struct_to_json(&s), json);
}

#[test]
fn observed_integers_deserialize_into_typed_structs() {
    #[derive(serde::Deserialize)]
    struct Spec {
        replicas: i64,
    }

    // Crossplane converts resources through structpb, which stores all
    // numbers as f64: replicas: 3 arrives as 3.0 on the wire.
    let req: RunFunctionRequest = serde_json::from_str(
        r#"{"observed": {"composite": {"resource": {"spec": {"replicas": 3.0}}}}}"#,
    )
    .unwrap();

    let xr = req.observed.unwrap().composite.unwrap().resource.unwrap();
    let spec: Spec = serde_json::from_value(resource::struct_to_json(&xr)["spec"].clone()).unwrap();
    assert_eq!(spec.replicas, 3);
}

#[test]
fn response_serializes_to_protojson() {
    let req = fixture();
    let mut rsp = response::to(&req, Duration::from_secs(90));
    response::normal(&mut rsp, "created bucket");

    let json = serde_json::to_value(&rsp).unwrap();
    assert_eq!(json["meta"]["tag"], "test-tag");
    assert_eq!(json["meta"]["ttl"], "90s");
    assert_eq!(json["results"][0]["severity"], "SEVERITY_NORMAL");
}
