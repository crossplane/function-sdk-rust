use std::collections::HashMap;

use function_sdk_rust::proto::v1::{
    Ready, Resource, RunFunctionRequest, RunFunctionResponse, State,
};
use function_sdk_rust::proto::{FILE_DESCRIPTOR_SET, UnknownFields};
use function_sdk_rust::{resource, response};
use prost::Message;
use prost::encoding;
use prost_types::FileDescriptorSet;
use prost_types::field_descriptor_proto::Type;

/// What prost generates for `State` from the vendored proto.
#[derive(Clone, PartialEq, Message)]
struct GeneratedState {
    #[prost(message, optional, tag = "1")]
    composite: Option<GeneratedResource>,
    #[prost(map = "string, message", tag = "2")]
    resources: HashMap<String, GeneratedResource>,
}

/// What prost generates for `Resource` from the vendored proto.
#[derive(Clone, PartialEq, Message)]
struct GeneratedResource {
    #[prost(message, optional, tag = "1")]
    resource: Option<pbjson_types::Struct>,
    #[prost(map = "string, bytes", tag = "2")]
    connection_details: HashMap<String, Vec<u8>>,
    #[prost(enumeration = "Ready", tag = "3")]
    ready: i32,
}

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
fn known_fields_encode_as_prost_generates_them() {
    let state = known_state();
    let generated = GeneratedState::decode(state.encode_to_vec().as_slice())
        .expect("the generated message decodes hand-written output");

    assert_eq!(state.encode_to_vec(), generated.encode_to_vec());
    assert_eq!(state.encoded_len(), generated.encoded_len());
    assert_eq!(
        State::decode(generated.encode_to_vec().as_slice()).expect("decodes"),
        state
    );
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

/// Feeds both messages every known field number with every wire type and a
/// few payloads, and expects the same verdict as the messages prost
/// generates. The generated messages drop unknown fields, so decoding the
/// hand-written output with them compares the known fields alone.
#[test]
fn malformed_input_decodes_as_prost_generates_it() {
    let payloads: [&[u8]; 6] = [
        &[],
        &[0x00],
        &[0x01],
        &[0x04, 0x0a, 0x02, b'h', b'i'],
        &[0x00, 0x00, 0x00, 0x00],
        &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    ];
    for tag in 1..=4u8 {
        for wire_type in 0..=5u8 {
            for payload in payloads {
                let mut input = vec![tag << 3 | wire_type];
                input.extend(payload);

                let state = State::decode(input.as_slice());
                let generated = GeneratedState::decode(input.as_slice());
                assert_eq!(state.is_ok(), generated.is_ok(), "State {input:?}");
                if let (Ok(state), Ok(generated)) = (state, generated) {
                    let known = GeneratedState::decode(state.encode_to_vec().as_slice())
                        .expect("the generated message decodes hand-written output");
                    assert_eq!(known, generated, "State {input:?}");
                }

                let resource = Resource::decode(input.as_slice());
                let generated = GeneratedResource::decode(input.as_slice());
                assert_eq!(resource.is_ok(), generated.is_ok(), "Resource {input:?}");
                if let (Ok(resource), Ok(generated)) = (resource, generated) {
                    let known = GeneratedResource::decode(resource.encode_to_vec().as_slice())
                        .expect("the generated message decodes hand-written output");
                    assert_eq!(known, generated, "Resource {input:?}");
                }
            }
        }
    }
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
fn debug_prints_ready_as_prost_generates_it() {
    for ready in [0, 1, 2, 7] {
        let resource = Resource {
            ready,
            ..Default::default()
        };
        let generated = GeneratedResource {
            ready,
            ..Default::default()
        };
        let want = format!("{generated:?}")
            .replace("GeneratedResource", "Resource")
            .replace(" }", ", unknown_fields: UnknownFields([]) }");
        assert_eq!(format!("{resource:?}"), want);
    }
    let ready = Resource {
        ready: Ready::True as i32,
        ..Default::default()
    };
    assert!(format!("{ready:?}").contains("ready: True"));
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

/// The hand-written messages must be updated when the vendored proto
/// changes them. Until then a new field still passes through as an unknown
/// field, but functions cannot read or set it.
#[test]
fn hand_written_messages_cover_every_field_of_the_proto() {
    let cases = [
        (
            "State",
            vec![
                ("composite", 1, Type::Message),
                ("resources", 2, Type::Message),
            ],
        ),
        (
            "Resource",
            vec![
                ("resource", 1, Type::Message),
                ("connection_details", 2, Type::Message),
                ("ready", 3, Type::Enum),
            ],
        ),
    ];

    let files = FileDescriptorSet::decode(FILE_DESCRIPTOR_SET).expect("the descriptor decodes");
    for (name, want) in cases {
        let message = files
            .file
            .iter()
            .filter(|file| file.package() == "apiextensions.fn.proto.v1")
            .flat_map(|file| &file.message_type)
            .find(|message| message.name() == name)
            .unwrap_or_else(|| panic!("{name} is in the descriptor"));
        let got: Vec<_> = message
            .field
            .iter()
            .map(|field| (field.name(), field.number(), field.r#type()))
            .collect();
        assert_eq!(got, want, "{name}: update sdk/src/proto/state.rs");
    }
}
