//! Holds every message codegen writes to the message prost derives from the
//! same proto.
//!
//! Codegen replaces prost's `Message` derive so that a message keeps unknown
//! fields (`codegen/src/messages.rs`), and writes what prost-build generated,
//! derive and all, to `tests/generated/prost_v1.rs`. The two must agree on
//! everything but unknown fields: the bytes of the known fields, the verdict
//! and the error on malformed input, `Debug`, the defaults and the accessors.
//! A map in a fixture has one entry, because prost encodes a map in hash
//! order and the tests compare bytes.
#![allow(deprecated)]

use std::collections::HashMap;
use std::fmt::Debug;

use function_sdk_rust::proto::FILE_DESCRIPTOR_SET;
use function_sdk_rust::proto::v1 as ours;
use function_sdk_rust::resource::json_to_struct;
use prost::Message;
use prost::encoding;
use prost_types::FileDescriptorSet;

/// The messages as prost derives them, which drop unknown fields.
#[allow(dead_code, clippy::all, clippy::pedantic)]
mod theirs {
    include!("generated/prost_v1.rs");
}

/// Runs `$check!(Message)` for every message of the function API.
/// `the_checks_cover_every_message_of_the_proto` holds the list to the
/// descriptor.
macro_rules! for_every_message {
    ($check:ident) => {
        $check!(RunFunctionRequest);
        $check!(Credentials);
        $check!(CredentialData);
        $check!(Resources);
        $check!(RunFunctionResponse);
        $check!(RequestMeta);
        $check!(Requirements);
        $check!(SchemaSelector);
        $check!(Schema);
        $check!(ResourceSelector);
        $check!(MatchLabels);
        $check!(ResponseMeta);
        $check!(State);
        $check!(Result);
        $check!(Resource);
        $check!(Condition);
    };
}

fn object() -> pbjson_types::Struct {
    let json = serde_json::json!({"spec": {"forProvider": {"region": "eu-central-1"}}});
    json_to_struct(json.as_object().expect("the fixture is an object"))
}

fn resource() -> theirs::Resource {
    theirs::Resource {
        resource: Some(object()),
        connection_details: HashMap::from([("password".to_string(), b"secret".to_vec())]),
        ready: theirs::Ready::True as i32,
    }
}

fn state() -> theirs::State {
    theirs::State {
        composite: Some(resource()),
        resources: HashMap::from([("bucket".to_string(), resource())]),
    }
}

fn resources() -> theirs::Resources {
    theirs::Resources {
        items: vec![resource(), theirs::Resource::default()],
    }
}

fn match_labels() -> theirs::MatchLabels {
    theirs::MatchLabels {
        labels: HashMap::from([("app".to_string(), "web".to_string())]),
    }
}

fn selector() -> theirs::ResourceSelector {
    theirs::ResourceSelector {
        api_version: "v1".to_string(),
        kind: "Secret".to_string(),
        namespace: Some("crossplane-system".to_string()),
        r#match: Some(theirs::resource_selector::Match::MatchLabels(match_labels())),
    }
}

fn credential_data() -> theirs::CredentialData {
    theirs::CredentialData {
        data: HashMap::from([("token".to_string(), b"t0ken".to_vec())]),
    }
}

fn credentials() -> theirs::Credentials {
    theirs::Credentials {
        source: Some(theirs::credentials::Source::CredentialData(
            credential_data(),
        )),
    }
}

fn schema() -> theirs::Schema {
    theirs::Schema {
        openapi_v3: Some(object()),
    }
}

fn schema_selector() -> theirs::SchemaSelector {
    theirs::SchemaSelector {
        api_version: "example.org/v1".to_string(),
        kind: "XBucket".to_string(),
    }
}

fn request_meta() -> theirs::RequestMeta {
    theirs::RequestMeta {
        tag: "tag".to_string(),
        capabilities: vec![
            theirs::Capability::Capabilities as i32,
            theirs::Capability::RequiredSchemas as i32,
            // A capability this proto does not name.
            42,
        ],
    }
}

fn response_meta() -> theirs::ResponseMeta {
    theirs::ResponseMeta {
        tag: "tag".to_string(),
        ttl: Some(pbjson_types::Duration {
            seconds: 60,
            nanos: 5,
        }),
    }
}

fn result() -> theirs::Result {
    theirs::Result {
        severity: theirs::Severity::Warning as i32,
        message: "a message".to_string(),
        reason: Some("AReason".to_string()),
        target: Some(theirs::Target::CompositeAndClaim as i32),
    }
}

fn condition() -> theirs::Condition {
    theirs::Condition {
        r#type: "DatabaseReady".to_string(),
        status: theirs::Status::ConditionFalse as i32,
        reason: "Creating".to_string(),
        message: Some("still creating".to_string()),
        target: Some(theirs::Target::Composite as i32),
    }
}

fn requirements() -> theirs::Requirements {
    theirs::Requirements {
        extra_resources: HashMap::from([(
            "by-name".to_string(),
            theirs::ResourceSelector {
                api_version: "v1".to_string(),
                kind: "ConfigMap".to_string(),
                namespace: None,
                r#match: Some(theirs::resource_selector::Match::MatchName(
                    "cm".to_string(),
                )),
            },
        )]),
        resources: HashMap::from([("by-labels".to_string(), selector())]),
        schemas: HashMap::from([("xbucket".to_string(), schema_selector())]),
    }
}

fn request() -> theirs::RunFunctionRequest {
    theirs::RunFunctionRequest {
        meta: Some(request_meta()),
        observed: Some(state()),
        desired: Some(state()),
        input: Some(object()),
        context: Some(object()),
        extra_resources: HashMap::from([("extra".to_string(), resources())]),
        credentials: HashMap::from([("api".to_string(), credentials())]),
        required_resources: HashMap::from([("required".to_string(), resources())]),
        required_schemas: HashMap::from([("xbucket".to_string(), schema())]),
    }
}

fn response() -> theirs::RunFunctionResponse {
    theirs::RunFunctionResponse {
        meta: Some(response_meta()),
        desired: Some(state()),
        results: vec![result(), theirs::Result::default()],
        context: Some(object()),
        requirements: Some(requirements()),
        conditions: vec![condition()],
        output: Some(object()),
    }
}

/// One value of the message with every field set, encoded by prost's
/// derive: what a valid message looks like on the wire.
fn sample(message: &str) -> Vec<u8> {
    match message {
        "RunFunctionRequest" => request().encode_to_vec(),
        "Credentials" => credentials().encode_to_vec(),
        "CredentialData" => credential_data().encode_to_vec(),
        "Resources" => resources().encode_to_vec(),
        "RunFunctionResponse" => response().encode_to_vec(),
        "RequestMeta" => request_meta().encode_to_vec(),
        "Requirements" => requirements().encode_to_vec(),
        "SchemaSelector" => schema_selector().encode_to_vec(),
        "Schema" => schema().encode_to_vec(),
        "ResourceSelector" => selector().encode_to_vec(),
        "MatchLabels" => match_labels().encode_to_vec(),
        "ResponseMeta" => response_meta().encode_to_vec(),
        "State" => state().encode_to_vec(),
        "Result" => result().encode_to_vec(),
        "Resource" => resource().encode_to_vec(),
        "Condition" => condition().encode_to_vec(),
        other => panic!("{other} has no sample: add one, and the message to for_every_message"),
    }
}

/// One field of every wire type proto3 can produce, under field numbers no
/// message of the function API uses.
fn fields_from_a_newer_crossplane() -> Vec<u8> {
    let mut buf = Vec::new();
    encoding::uint64::encode(100, &u64::MAX, &mut buf);
    encoding::fixed64::encode(101, &7, &mut buf);
    encoding::fixed32::encode(102, &9, &mut buf);
    encoding::string::encode(103, &"from-the-future".to_string(), &mut buf);
    buf
}

/// `Debug` of one of our messages without the field prost's does not have.
fn debug_of_known_fields(message: &impl Debug) -> String {
    format!("{message:?}").replace(", unknown_fields: UnknownFields([])", "")
}

/// Decodes `input` as both messages and expects one verdict: the same
/// error, or the same known fields. prost's message drops unknown fields,
/// so decoding our output with it compares the known fields alone.
fn agree<Ours, Theirs>(message: &str, input: &[u8])
where
    Ours: Message + Default + Debug,
    Theirs: Message + Default + Debug + PartialEq,
{
    match (Ours::decode(input), Theirs::decode(input)) {
        (Ok(ours), Ok(theirs)) => {
            let encoded = ours.encode_to_vec();
            assert_eq!(ours.encoded_len(), encoded.len(), "{message} {input:?}");
            let known = Theirs::decode(encoded.as_slice())
                .unwrap_or_else(|e| panic!("{message} {input:?}: prost refuses our output: {e}"));
            assert_eq!(known, theirs, "{message} {input:?}");
        }
        (Err(ours), Err(theirs)) => {
            assert_eq!(ours.to_string(), theirs.to_string(), "{message} {input:?}");
        }
        (ours, theirs) => panic!("{message} {input:?}: ours {ours:?}, prost's {theirs:?}"),
    }
}

fn a_valid_message_is_the_same_bytes<Ours, Theirs>(message: &str)
where
    Ours: Message + Default + Debug,
    Theirs: Message + Default + Debug + PartialEq,
{
    let sample = sample(message);
    let ours = Ours::decode(sample.as_slice()).expect("our message decodes prost's output");
    let theirs = Theirs::decode(sample.as_slice()).expect("prost decodes its own output");

    assert_eq!(ours.encode_to_vec(), sample, "{message}");
    assert_eq!(ours.encoded_len(), sample.len(), "{message}");
    assert_eq!(
        debug_of_known_fields(&ours),
        format!("{theirs:?}"),
        "{message}"
    );
    agree::<Ours, Theirs>(message, &sample);

    // Cut anywhere, the sample is the same error or the same shorter message.
    for end in 0..sample.len() {
        agree::<Ours, Theirs>(message, &sample[..end]);
    }
}

fn a_default_message_is_prost_s_default<Ours, Theirs>(message: &str)
where
    Ours: Message + Default + Debug,
    Theirs: Message + Default + Debug,
{
    assert!(Ours::default().encode_to_vec().is_empty(), "{message}");
    assert_eq!(
        debug_of_known_fields(&Ours::default()),
        format!("{:?}", Theirs::default()),
        "{message}"
    );

    let mut cleared = Ours::decode(sample(message).as_slice()).expect("decodes");
    cleared.clear();
    assert!(cleared.encode_to_vec().is_empty(), "{message}");
}

/// Every field number the proto uses and a few it does not, with every wire
/// type and a few payloads.
fn malformed_input_is_prost_s_verdict<Ours, Theirs>(message: &str)
where
    Ours: Message + Default + Debug,
    Theirs: Message + Default + Debug + PartialEq,
{
    let payloads: [&[u8]; 8] = [
        &[],
        &[0x00],
        &[0x01],
        &[0x7f],
        &[0x04, 0x0a, 0x02, b'h', b'i'],
        &[0x02, 0x08, 0x01],
        &[0x00, 0x00, 0x00, 0x00],
        &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    ];
    for tag in 1..=12u8 {
        for wire_type in 0..=5u8 {
            for payload in payloads {
                let mut input = vec![tag << 3 | wire_type];
                input.extend(payload);
                agree::<Ours, Theirs>(message, &input);
            }
        }
    }
}

fn unknown_fields_are_kept_byte_for_byte<Ours, Theirs>(message: &str)
where
    Ours: Message + Default + Debug,
    Theirs: Message + Default + Debug,
{
    let sample = sample(message);
    let mut newer = sample.clone();
    newer.extend(fields_from_a_newer_crossplane());

    let ours = Ours::decode(newer.as_slice()).expect("decodes");
    assert_eq!(ours.encoded_len(), newer.len(), "{message}");
    assert_eq!(ours.encode_to_vec(), newer, "{message}");
    assert!(
        format!("{ours:?}").contains("unknown_fields: UnknownFields([160, 6,"),
        "{message}: {ours:?}"
    );

    // What the rewrite is for: prost's own message loses them.
    let theirs = Theirs::decode(newer.as_slice()).expect("decodes");
    assert_eq!(theirs.encode_to_vec(), sample, "{message}");

    // An unknown field cut short is an error, not a shorter message.
    newer.truncate(newer.len() - 1);
    assert!(Ours::decode(newer.as_slice()).is_err(), "{message}");
}

#[test]
fn a_valid_message_encodes_as_prost_encodes_it() {
    macro_rules! check {
        ($message:ident) => {
            a_valid_message_is_the_same_bytes::<ours::$message, theirs::$message>(stringify!(
                $message
            ))
        };
    }
    for_every_message!(check);
}

#[test]
fn a_default_message_is_what_prost_defaults_to() {
    macro_rules! check {
        ($message:ident) => {
            a_default_message_is_prost_s_default::<ours::$message, theirs::$message>(stringify!(
                $message
            ))
        };
    }
    for_every_message!(check);
}

#[test]
fn malformed_input_decodes_as_prost_decodes_it() {
    macro_rules! check {
        ($message:ident) => {
            malformed_input_is_prost_s_verdict::<ours::$message, theirs::$message>(stringify!(
                $message
            ))
        };
    }
    for_every_message!(check);
}

#[test]
fn every_message_keeps_unknown_fields() {
    macro_rules! check {
        ($message:ident) => {
            unknown_fields_are_kept_byte_for_byte::<ours::$message, theirs::$message>(stringify!(
                $message
            ))
        };
    }
    for_every_message!(check);
}

/// The accessors prost's derive adds: an enumeration read as its enum, set
/// and pushed, and an optional string read as a `&str`.
#[test]
fn the_accessors_are_prost_s() {
    for ready in [0, 1, 2, 7] {
        let (mut a, mut b) = (ours::Resource::default(), theirs::Resource::default());
        (a.ready, b.ready) = (ready, ready);
        assert_eq!(a.ready() as i32, b.ready() as i32, "ready {ready}");
        a.set_ready(ours::Ready::False);
        b.set_ready(theirs::Ready::False);
        assert_eq!(a.ready, b.ready);
    }

    let unset = (ours::Result::default(), theirs::Result::default());
    assert_eq!(unset.0.reason(), unset.1.reason());
    assert_eq!(unset.0.target() as i32, unset.1.target() as i32);
    assert_eq!(unset.0.severity() as i32, unset.1.severity() as i32);
    for value in [0, 2, 9] {
        let (mut a, mut b) = (ours::Result::default(), theirs::Result::default());
        (a.severity, b.severity) = (value, value);
        (a.target, b.target) = (Some(value), Some(value));
        (a.reason, b.reason) = (Some("why".to_string()), Some("why".to_string()));
        assert_eq!(a.severity() as i32, b.severity() as i32, "severity {value}");
        assert_eq!(a.target() as i32, b.target() as i32, "target {value}");
        assert_eq!(a.reason(), b.reason());
        a.set_severity(ours::Severity::Fatal);
        b.set_severity(theirs::Severity::Fatal);
        a.set_target(ours::Target::CompositeAndClaim);
        b.set_target(theirs::Target::CompositeAndClaim);
        assert_eq!((a.severity, a.target), (b.severity, b.target));
    }

    let (mut a, mut b) = (ours::Condition::default(), theirs::Condition::default());
    assert_eq!(a.message(), b.message());
    assert_eq!(a.status() as i32, b.status() as i32);
    assert_eq!(a.target() as i32, b.target() as i32);
    a.set_status(ours::Status::ConditionTrue);
    b.set_status(theirs::Status::ConditionTrue);
    a.set_target(ours::Target::Composite);
    b.set_target(theirs::Target::Composite);
    assert_eq!((a.status, a.target), (b.status, b.target));

    let (a, b) = (
        ours::ResourceSelector::default(),
        theirs::ResourceSelector::default(),
    );
    assert_eq!(a.namespace(), b.namespace());

    let (mut a, mut b) = (ours::RequestMeta::default(), theirs::RequestMeta::default());
    (a.capabilities, b.capabilities) = (vec![1, 42, 5], vec![1, 42, 5]);
    a.push_capabilities(ours::Capability::Conditions);
    b.push_capabilities(theirs::Capability::Conditions);
    assert_eq!(a.capabilities, b.capabilities);
    let read: (Vec<i32>, Vec<i32>) = (
        a.capabilities().map(|c| c as i32).collect(),
        b.capabilities().map(|c| c as i32).collect(),
    );
    assert_eq!(read.0, read.1);
    assert_eq!(read.0, [1, 5, 4]);
}

/// A message the proto gains has to join `for_every_message` and `sample`,
/// or nothing here would hold it to prost's.
#[test]
fn the_checks_cover_every_message_of_the_proto() {
    let mut checked: Vec<&str> = Vec::new();
    macro_rules! check {
        ($message:ident) => {
            checked.push(stringify!($message))
        };
    }
    for_every_message!(check);
    checked.sort_unstable();

    let files = FileDescriptorSet::decode(FILE_DESCRIPTOR_SET).expect("the descriptor decodes");
    let mut messages = Vec::new();
    for message in files
        .file
        .iter()
        .filter(|file| file.package() == "apiextensions.fn.proto.v1")
        .flat_map(|file| &file.message_type)
    {
        messages.push(message.name());
        // A map field is a nested entry message, which is no message of the
        // API. Any other nested message would need its own check.
        let nested: Vec<_> = message
            .nested_type
            .iter()
            .filter(|nested| !nested.options.as_ref().is_some_and(|o| o.map_entry()))
            .map(|nested| nested.name())
            .collect();
        assert!(nested.is_empty(), "{} nests {nested:?}", message.name());
    }
    messages.sort_unstable();
    assert_eq!(checked, messages);
}
