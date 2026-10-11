//! Helpers for working with RunFunctionRequests.

use crate::Error;
use crate::proto::v1::{Capability, CredentialData, Resource, RunFunctionRequest, credentials};
use crate::resource::{pb_to_json, struct_to_json};

/// The requirement name a WatchOperation uses to inject the watched resource.
pub const WATCHED_RESOURCE_KEY: &str = "ops.crossplane.io/watched-resource";

/// Gets the function's input from the request as a typed value.
///
/// Input is the function-defined config from the Composition's
/// `spec.pipeline[].input` block. Crossplane never validates it, so
/// functions must - deserializing into a typed struct is the first line of
/// that validation. Returns [`Error::MissingInput`] when the pipeline step
/// has no input at all.
pub fn get_input<T: serde::de::DeserializeOwned>(req: &RunFunctionRequest) -> Result<T, Error> {
    let input = req.input.as_ref().ok_or(Error::MissingInput)?;
    Ok(serde_json::from_value(struct_to_json(input))?)
}

/// Gets the observed composite resource (XR), if the request carries one.
///
/// Read it as a typed value with [`crate::resource::get`].
pub fn observed_composite(req: &RunFunctionRequest) -> Option<&Resource> {
    req.observed.as_ref()?.composite.as_ref()
}

/// Gets an observed composed resource by name.
///
/// Returns None when Crossplane has not observed the resource, for example
/// before it is first created. Read it as a typed value with
/// [`crate::resource::get`].
pub fn observed_composed<'a>(req: &'a RunFunctionRequest, name: &str) -> Option<&'a Resource> {
    req.observed.as_ref()?.resources.get(name)
}

/// Gets a function context value by key, if present.
///
/// See [`crate::context`] for well-known context keys.
pub fn get_context_key(req: &RunFunctionRequest, key: &str) -> Option<serde_json::Value> {
    req.context.as_ref()?.fields.get(key).map(pb_to_json)
}

/// Gets required resources by requirement name from the request.
///
/// Returns None when Crossplane has not (yet) resolved the requirement, and
/// an empty Vec when it resolved the requirement but found no matches.
/// Always declare requirements with [`crate::response::require_resources`];
/// Crossplane considers them satisfied when they stabilize across calls.
pub fn get_required_resources(
    req: &RunFunctionRequest,
    name: &str,
) -> Option<Vec<serde_json::Value>> {
    let resources = req.required_resources.get(name)?;
    Some(
        resources
            .items
            .iter()
            .filter_map(|r| r.resource.as_ref())
            .map(struct_to_json)
            .collect(),
    )
}

/// Gets a single required resource by requirement name from the request.
///
/// A convenience for requirements that match exactly one resource. Returns
/// None when the requirement is unresolved or matched nothing.
pub fn get_required_resource(req: &RunFunctionRequest, name: &str) -> Option<serde_json::Value> {
    get_required_resources(req, name)?.into_iter().next()
}

/// Gets required resources by requirement name as typed values.
///
/// Returns `Ok(None)` when Crossplane has not (yet) resolved the
/// requirement, and an empty Vec when it resolved the requirement but found
/// no matches. Integral numbers are restored to integers first, as in
/// [`crate::resource::get`], so integer fields of generated models
/// deserialize.
pub fn get_required_resources_as<T: serde::de::DeserializeOwned>(
    req: &RunFunctionRequest,
    name: &str,
) -> Result<Option<Vec<T>>, Error> {
    get_required_resources(req, name)
        .map(|resources| resources.into_iter().map(serde_json::from_value).collect())
        .transpose()
        .map_err(Error::from)
}

/// Gets a single required resource by requirement name as a typed value.
///
/// Returns `Ok(None)` when the requirement is unresolved or matched nothing.
pub fn get_required_resource_as<T: serde::de::DeserializeOwned>(
    req: &RunFunctionRequest,
    name: &str,
) -> Result<Option<T>, Error> {
    get_required_resource(req, name)
        .map(serde_json::from_value)
        .transpose()
        .map_err(Error::from)
}

/// Gets the watched resource that triggered this operation, if any.
///
/// When a WatchOperation creates an Operation it injects the resource that
/// changed under the requirement name [`WATCHED_RESOURCE_KEY`].
pub fn get_watched_resource(req: &RunFunctionRequest) -> Option<serde_json::Value> {
    get_required_resource(req, WATCHED_RESOURCE_KEY)
}

/// Gets the supplied credential data from the request, if any.
pub fn get_credential_data<'a>(
    req: &'a RunFunctionRequest,
    name: &str,
) -> Option<&'a CredentialData> {
    match req.credentials.get(name)?.source.as_ref()? {
        credentials::Source::CredentialData(data) => Some(data),
    }
}

/// Checks whether Crossplane advertises its capabilities at all.
///
/// Crossplane v2.2 and later advertise capabilities in the request metadata.
/// If this returns false the calling Crossplane predates capability
/// advertisement, and [`has_capability`] returns false even for features
/// that older Crossplane does support.
pub fn advertises_capabilities(req: &RunFunctionRequest) -> bool {
    has_capability(req, Capability::Capabilities)
}

/// Checks whether Crossplane advertises a particular capability.
pub fn has_capability(req: &RunFunctionRequest, cap: Capability) -> bool {
    req.meta
        .as_ref()
        .is_some_and(|m| m.capabilities().any(|c| c == cap))
}

/// Gets a required OpenAPI v3 schema by requirement name from the request.
///
/// Returns None both when the requirement is unresolved and when Crossplane
/// resolved it but found no schema. To distinguish the two, check
/// `req.required_schemas.contains_key(name)`.
pub fn get_required_schema(req: &RunFunctionRequest, name: &str) -> Option<serde_json::Value> {
    req.required_schemas
        .get(name)?
        .openapi_v3
        .as_ref()
        .map(struct_to_json)
}
