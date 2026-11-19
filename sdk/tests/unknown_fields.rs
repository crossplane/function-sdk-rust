//! What a function sees of the fields its SDK does not know: they pass
//! through it, on every message. `generated_messages.rs` holds the messages
//! themselves to the ones prost derives.

use std::collections::HashMap;

use function_sdk_rust::proto::UnknownFields;
use function_sdk_rust::proto::v1::{
    Ready, RequestMeta, Requirements, Resource, ResourceSelector, ResponseMeta, Result as FnResult,
    RunFunctionRequest, RunFunctionResponse, State,
};
use function_sdk_rust::{resource, response};
use prost::Message;
use prost::encoding;

/// Every map in the fixtures has one entry, because prost encodes maps in
/// hash order and the tests compare encoded bytes.
fn bucket() -> pbjson_types::Struct {
    let json = serde_json::json!({"spec": {"forProvider": {"region": "eu-central-1"}}});
    resource::json_to_struct(json.as_object().expect("the fixture is an object"))
}

fn known_resource() -> Resource {
    Resource {
        resource: Some(bucket()),
        connection_details: HashMap::from([("password".to_string(), b"secret".to_vec())]),
        ready: Ready::True as i32,
        ..Default::default()
    }
}

fn known_state() -> State {
    State {
        composite: Some(known_resource()),
        resources: HashMap::from([("bucket".to_string(), known_resource())]),
        ..Default::default()
    }
}

/// Encodes one field of every wire type proto3 can produce, using field
/// numbers no message of the function API uses.
fn fields_from_a_newer_crossplane() -> Vec<u8> {
    let mut buf = Vec::new();
    encoding::uint64::encode(100, &u64::MAX, &mut buf);
    encoding::fixed64::encode(101, &7, &mut buf);
    encoding::fixed32::encode(102, &9, &mut buf);
    encoding::string::encode(103, &"from-the-future".to_string(), &mut buf);
    buf
}

fn with_unknown_fields(message: &impl Message) -> Vec<u8> {
    let mut buf = message.encode_to_vec();
    buf.extend(fields_from_a_newer_crossplane());
    buf
}

/// A desired State as a newer Crossplane might send it, with fields this
/// SDK does not know on the State and on its one composed resource.
fn newer_state() -> Vec<u8> {
    let mut state = Vec::new();
    encoding::hash_map::encode(
        encoding::string::encode,
        encoding::string::encoded_len,
        encoding::bytes::encode,
        encoding::bytes::encoded_len,
        2,
        &HashMap::from([("bucket".to_string(), with_unknown_fields(&known_resource()))]),
        &mut state,
    );
    state.extend(fields_from_a_newer_crossplane());
    state
}

fn request_with_desired(state: &[u8]) -> RunFunctionRequest {
    let mut req = Vec::new();
    encoding::bytes::encode(3, &state.to_vec(), &mut req);
    RunFunctionRequest::decode(req.as_slice()).expect("the request decodes")
}

fn desired_bytes(rsp: &RunFunctionResponse) -> Vec<u8> {
    rsp.desired
        .as_ref()
        .expect("the response has desired state")
        .encode_to_vec()
}

#[test]
fn default_messages_encode_to_nothing() {
    assert!(State::default().encode_to_vec().is_empty());
    assert!(Resource::default().encode_to_vec().is_empty());
}

#[test]
fn unknown_fields_round_trip_byte_for_byte() {
    let resource = with_unknown_fields(&known_resource());
    let decoded = Resource::decode(resource.as_slice()).expect("decodes");
    assert_ne!(decoded.unknown_fields, UnknownFields::default());
    assert_eq!(decoded.encoded_len(), resource.len());
    assert_eq!(decoded.encode_to_vec(), resource);

    let state = newer_state();
    let decoded = State::decode(state.as_slice()).expect("decodes");
    assert_ne!(decoded.unknown_fields, UnknownFields::default());
    assert_eq!(decoded.encoded_len(), state.len());
    assert_eq!(decoded.encode_to_vec(), state);
}

#[test]
fn unknown_fields_do_not_disturb_known_fields() {
    let decoded =
        Resource::decode(with_unknown_fields(&known_resource()).as_slice()).expect("decodes");
    let known = Resource {
        unknown_fields: UnknownFields::default(),
        ..decoded
    };
    assert_eq!(known, known_resource());
}

#[test]
fn response_to_keeps_unknown_fields_of_desired_state() {
    let req = request_with_desired(&newer_state());
    let rsp = response::to(&req, response::DEFAULT_TTL);
    let rsp = RunFunctionResponse::decode(rsp.encode_to_vec().as_slice()).expect("decodes");

    assert_eq!(desired_bytes(&rsp), newer_state());
}

#[test]
fn modifying_a_resource_in_place_keeps_its_unknown_fields() {
    let req = request_with_desired(&newer_state());
    let mut rsp = response::to(&req, response::DEFAULT_TTL);
    resource::set_ready(response::desired_composed(&mut rsp, "bucket"), Ready::False);

    let desired = State::decode(desired_bytes(&rsp).as_slice()).expect("decodes");
    let bucket = &desired.resources["bucket"];
    assert_eq!(bucket.ready(), Ready::False);
    assert_ne!(bucket.unknown_fields, UnknownFields::default());
    assert_ne!(desired.unknown_fields, UnknownFields::default());
}

#[test]
fn replacing_a_resource_drops_its_unknown_fields() {
    let req = request_with_desired(&newer_state());
    let mut rsp = response::to(&req, response::DEFAULT_TTL);
    rsp.desired
        .as_mut()
        .expect("the response has desired state")
        .resources
        .insert("bucket".to_string(), known_resource());

    let desired = State::decode(desired_bytes(&rsp).as_slice()).expect("decodes");
    assert_eq!(desired.resources["bucket"], known_resource());
    assert_ne!(desired.unknown_fields, UnknownFields::default());
}

#[test]
fn a_truncated_unknown_field_fails_to_decode() {
    let mut resource = with_unknown_fields(&known_resource());
    resource.truncate(resource.len() - 1);
    assert!(Resource::decode(resource.as_slice()).is_err());
}

#[test]
fn a_known_field_with_the_wrong_wire_type_fails_to_decode() {
    let mut resource = Vec::new();
    encoding::fixed64::encode(3, &1, &mut resource);
    let error = Resource::decode(resource.as_slice()).expect_err("ready is a varint");
    assert!(error.to_string().contains("Resource.ready"), "{error}");
}

#[test]
fn a_map_field_with_the_wrong_wire_type_fails_to_decode() {
    let error = Resource::decode([0x10, 0x00].as_slice()).expect_err("a map is not a varint");
    assert!(
        error.to_string().contains("Resource.connection_details"),
        "{error}"
    );
    let error = State::decode([0x10, 0x00].as_slice()).expect_err("a map is not a varint");
    assert!(error.to_string().contains("State.resources"), "{error}");
}

#[test]
fn unknown_fields_of_the_composite_resource_are_kept() {
    let mut state = Vec::new();
    encoding::bytes::encode(1, &with_unknown_fields(&known_resource()), &mut state);

    let decoded = State::decode(state.as_slice()).expect("decodes");
    let composite = decoded.composite.as_ref().expect("has a composite");
    assert_ne!(composite.unknown_fields, UnknownFields::default());
    assert_eq!(decoded.encode_to_vec(), state);
}

#[test]
fn clear_drops_unknown_fields() {
    let mut resource =
        Resource::decode(with_unknown_fields(&known_resource()).as_slice()).expect("decodes");
    resource.clear();
    assert_eq!(resource, Resource::default());
}

#[test]
fn json_has_no_unknown_fields() {
    let state = State::decode(newer_state().as_slice()).expect("decodes");
    let json = serde_json::to_value(&state).expect("serializes");
    assert_eq!(
        json,
        serde_json::to_value(known_state_without_composite()).expect("serializes")
    );

    let from_json: State = serde_json::from_value(json).expect("deserializes");
    assert_eq!(from_json, known_state_without_composite());
}

fn known_state_without_composite() -> State {
    State {
        composite: None,
        ..known_state()
    }
}

/// `message` as field `number` of the message that holds it.
fn field(number: u32, message: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    encoding::bytes::encode(number, &message.to_vec(), &mut buf);
    buf
}

/// The fields Crossplane has added since the function API went v1 are on
/// the request, its meta, the response, the requirements and the resource
/// selector: none is on State or Resource. A function that forwards a
/// request, or returns a response it did not build, keeps what a newer
/// Crossplane or a newer function put on any of them.
#[test]
fn unknown_fields_pass_through_every_message_of_a_request_and_a_response() {
    let meta = RequestMeta {
        tag: "tag".to_string(),
        ..Default::default()
    };
    let mut request = field(1, &with_unknown_fields(&meta));
    request.extend(field(3, &newer_state()));
    request.extend(fields_from_a_newer_crossplane());

    let decoded = RunFunctionRequest::decode(request.as_slice()).expect("decodes");
    assert_ne!(decoded.unknown_fields, UnknownFields::default());
    let decoded_meta = decoded.meta.as_ref().expect("has meta");
    assert_ne!(decoded_meta.unknown_fields, UnknownFields::default());
    assert_eq!(decoded_meta.tag, "tag");
    assert_eq!(decoded.encode_to_vec(), request);

    let selector = ResourceSelector {
        api_version: "v1".to_string(),
        kind: "Secret".to_string(),
        ..Default::default()
    };
    let mut selectors = Vec::new();
    encoding::hash_map::encode(
        encoding::string::encode,
        encoding::string::encoded_len,
        encoding::bytes::encode,
        encoding::bytes::encoded_len,
        2,
        &HashMap::from([("secret".to_string(), with_unknown_fields(&selector))]),
        &mut selectors,
    );
    selectors.extend(fields_from_a_newer_crossplane());
    let result = FnResult {
        message: "a message".to_string(),
        ..Default::default()
    };
    let response_meta = ResponseMeta {
        tag: "tag".to_string(),
        ..Default::default()
    };
    let mut response = field(1, &with_unknown_fields(&response_meta));
    response.extend(field(2, &newer_state()));
    response.extend(field(3, &with_unknown_fields(&result)));
    response.extend(field(5, &selectors));
    response.extend(fields_from_a_newer_crossplane());

    let decoded = RunFunctionResponse::decode(response.as_slice()).expect("decodes");
    assert_ne!(decoded.unknown_fields, UnknownFields::default());
    assert_ne!(
        decoded.meta.as_ref().expect("has meta").unknown_fields,
        UnknownFields::default()
    );
    assert_ne!(decoded.results[0].unknown_fields, UnknownFields::default());
    let requirements: &Requirements = decoded.requirements.as_ref().expect("has requirements");
    assert_ne!(requirements.unknown_fields, UnknownFields::default());
    assert_ne!(
        requirements.resources["secret"].unknown_fields,
        UnknownFields::default()
    );
    assert_eq!(requirements.resources["secret"].kind, "Secret");
    assert_eq!(decoded.encode_to_vec(), response);
}

#[test]
fn debug_prints_an_enumeration_by_name_and_the_unknown_fields() {
    let ready = Resource {
        ready: Ready::True as i32,
        ..Default::default()
    };
    assert_eq!(
        format!("{ready:?}"),
        "Resource { resource: None, connection_details: {}, ready: True, unknown_fields: UnknownFields([]) }"
    );
}
