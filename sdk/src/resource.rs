//! Helpers for working with composite and composed resources.

use pbjson_types::{ListValue, Struct, Value, value::Kind};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::Error;
use crate::proto::v1::{Ready, Resource};

/// Converts a protobuf Struct to a JSON value.
///
/// The conversion is total: a number that JSON cannot represent (NaN or
/// infinity) becomes null. Struct numbers are f64, so integers above 2^53
/// have already lost precision on the wire. Integral numbers are emitted as
/// JSON integers (see [`pb_to_json`]), so the result deserializes cleanly
/// into typed structs with integer fields.
pub fn struct_to_json(s: &Struct) -> serde_json::Value {
    serde_json::Value::Object(
        s.fields
            .iter()
            .map(|(k, v)| (k.clone(), pb_to_json(v)))
            .collect(),
    )
}

/// Deserializes a resource's JSON representation into a typed value.
///
/// Works for both the composite resource (XR) and composed resources - both
/// are represented by [`Resource`]. Pass
/// [`request::observed_composite`](crate::request::observed_composite) for
/// the observed XR, or
/// [`request::observed_composed`](crate::request::observed_composed) for an
/// observed composed resource; desired resources work the same way. Returns
/// [`Error::MissingResource`] when `resource` is `None` or has no JSON
/// representation set, for example a composed resource Crossplane has not
/// yet observed.
pub fn get<T: serde::de::DeserializeOwned>(resource: Option<&Resource>) -> Result<T, Error> {
    let s = resource
        .and_then(|r| r.resource.as_ref())
        .ok_or(Error::MissingResource)?;
    Ok(serde_json::from_value(struct_to_json(s))?)
}

/// Converts a JSON object to a protobuf Struct.
pub fn json_to_struct(m: &serde_json::Map<String, serde_json::Value>) -> Struct {
    Struct {
        fields: m.iter().map(|(k, v)| (k.clone(), json_to_pb(v))).collect(),
    }
}

/// Converts a protobuf Value to a JSON value. NaN and infinity become null.
///
/// Struct erases the integer-ness of every number: Crossplane converts
/// resources through structpb, which stores all numbers as f64, so an
/// observed `replicas: 3` arrives as 3.0. Serde refuses to deserialize a
/// float into an integer field, so integral numbers are restored to JSON
/// integers here. K8s resources have no genuinely float-typed fields this
/// could misrepresent.
pub fn pb_to_json(v: &Value) -> serde_json::Value {
    match &v.kind {
        None | Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::NumberValue(n)) => number_to_json(*n),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s.clone()),
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(*b),
        Some(Kind::StructValue(s)) => struct_to_json(s),
        Some(Kind::ListValue(l)) => {
            serde_json::Value::Array(l.values.iter().map(pb_to_json).collect())
        }
    }
}

fn number_to_json(n: f64) -> serde_json::Value {
    const SAFE_INT: f64 = (1i64 << 53) as f64;
    if n.fract() == 0.0 && n.abs() <= SAFE_INT {
        return serde_json::Value::Number(serde_json::Number::from(n as i64));
    }
    serde_json::Number::from_f64(n)
        .map(serde_json::Value::Number)
        .unwrap_or(serde_json::Value::Null)
}

/// Converts a JSON value to a protobuf Value. Numbers become f64.
pub fn json_to_pb(v: &serde_json::Value) -> Value {
    let kind = match v {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(*b),
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(f64::NAN)),
        serde_json::Value::String(s) => Kind::StringValue(s.clone()),
        serde_json::Value::Array(a) => Kind::ListValue(ListValue {
            values: a.iter().map(json_to_pb).collect(),
        }),
        serde_json::Value::Object(m) => Kind::StructValue(json_to_struct(m)),
    };
    Value { kind: Some(kind) }
}

/// Updates a composite or composed resource from any serializable source.
///
/// The source must serialize to a JSON object, for example a
/// `serde_json::json!` literal or a typed struct deriving Serialize. Top
/// level fields that already exist are overwritten and fields that do not
/// exist are added, like a map update. Include only the fields this function
/// has an opinion about: Crossplane treats desired state as server-side
/// apply intent.
pub fn update<T: Serialize + ?Sized>(r: &mut Resource, source: &T) -> Result<(), Error> {
    let value = serde_json::to_value(source)?;
    let serde_json::Value::Object(map) = value else {
        return Err(Error::NotAnObject);
    };
    let target = r.resource.get_or_insert_default();
    for (k, v) in &map {
        target.fields.insert(k.clone(), json_to_pb(v));
    }
    Ok(())
}

/// Updates a resource's status from any serializable source.
///
/// Equivalent to calling [`update`] with `{"status": source}`.
pub fn update_status<T: Serialize + ?Sized>(r: &mut Resource, status: &T) -> Result<(), Error> {
    update(
        r,
        &serde_json::json!({"status": serde_json::to_value(status)?}),
    )
}

/// Sets whether a desired resource should be considered ready.
///
/// Set `Ready::True` on a desired composed resource to mark it ready, or on
/// the desired XR to override Crossplane's standard readiness detection.
pub fn set_ready(r: &mut Resource, ready: Ready) {
    r.ready = ready as i32;
}

/// Adds a connection detail to a resource.
///
/// Only meaningful on the desired XR of legacy (v1) XRs; Crossplane ignores
/// desired connection details everywhere else.
pub fn add_connection_detail(r: &mut Resource, key: impl Into<String>, value: impl Into<Vec<u8>>) {
    r.connection_details.insert(key.into(), value.into());
}

/// A status condition of a resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Condition {
    /// Type of the condition, for example Ready.
    pub typ: String,
    /// Status of the condition: True, False, or Unknown.
    pub status: String,
    /// Machine-readable reason for the condition status, typically PascalCase.
    pub reason: Option<String>,
    /// Human-readable message.
    pub message: Option<String>,
    /// RFC 3339 time of the last status transition.
    pub last_transition_time: Option<String>,
}

impl Condition {
    fn unknown(typ: &str) -> Self {
        Self {
            typ: typ.to_string(),
            status: "Unknown".to_string(),
            reason: None,
            message: None,
            last_transition_time: None,
        }
    }
}

/// Gets the supplied status condition of the supplied resource.
///
/// A condition is always returned: if the resource is None or the condition
/// is not present, a condition with status Unknown is returned. Accepting an
/// Option makes it safe to pass the result of a map `get` directly, for
/// example `get_condition(req.observed.resources.get("bucket"), "Ready")`.
pub fn get_condition(resource: Option<&Resource>, typ: &str) -> Condition {
    let Some(s) = resource.and_then(|r| r.resource.as_ref()) else {
        return Condition::unknown(typ);
    };
    let Some(Kind::StructValue(status)) = s.fields.get("status").and_then(|v| v.kind.as_ref())
    else {
        return Condition::unknown(typ);
    };
    let Some(Kind::ListValue(conditions)) = status
        .fields
        .get("conditions")
        .and_then(|v| v.kind.as_ref())
    else {
        return Condition::unknown(typ);
    };

    for value in &conditions.values {
        let Some(Kind::StructValue(c)) = value.kind.as_ref() else {
            continue;
        };
        if get_str(c, "type") != Some(typ) {
            continue;
        }
        return Condition {
            typ: typ.to_string(),
            status: get_str(c, "status").unwrap_or("Unknown").to_string(),
            reason: get_str(c, "reason").map(String::from),
            message: get_str(c, "message").map(String::from),
            last_transition_time: get_str(c, "lastTransitionTime").map(String::from),
        };
    }

    Condition::unknown(typ)
}

fn get_str<'a>(s: &'a Struct, key: &str) -> Option<&'a str> {
    match s.fields.get(key)?.kind.as_ref()? {
        Kind::StringValue(v) => Some(v),
        _ => None,
    }
}

const DNS_LABEL_MAX: usize = 63;
const HASH_LEN: usize = 5;

/// Builds a deterministic, DNS-label-safe name for a child resource.
///
/// Joins the parts with the separator, appends a deterministic 5-character
/// hash suffix for uniqueness, and truncates the prefix so the result is at
/// most 63 characters. The hash is always appended, even for short names, so
/// names are visually consistent regardless of length.
pub fn child_name(parts: &[&str], sep: &str) -> String {
    let full = parts.join(sep);
    let digest = Sha256::digest(full.as_bytes());
    let hash: String = digest
        .iter()
        .take(HASH_LEN.div_ceil(2))
        .map(|b| format!("{b:02x}"))
        .collect();
    let hash = &hash[..HASH_LEN];
    let max_prefix = DNS_LABEL_MAX - HASH_LEN - sep.len();
    let prefix: String = full.chars().take(max_prefix).collect();
    let prefix = prefix.trim_end_matches(sep);
    format!("{prefix}{sep}{hash}")
}
