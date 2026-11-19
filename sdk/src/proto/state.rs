//! The messages a function carries forward from request to response.
//!
//! prost discards fields it does not know. [`State`] and [`Resource`] are
//! written by hand instead of generated so that fields a newer Crossplane
//! adds to them survive a function built against an older proto, as they do
//! in function-sdk-go. Known fields are decoded and encoded as prost
//! generates it for these messages.

use std::collections::HashMap;
use std::fmt;

use prost::bytes::{Buf, BufMut};
use prost::encoding::{self, DecodeContext, WireType};
use prost::{DecodeError, Message};

use super::v1::Ready;

/// Fields of a message that this SDK's proto does not know.
///
/// Crossplane may add fields to a message after this SDK was built. They
/// are kept when a message is decoded and written back when it is encoded,
/// so a function passes them down the pipeline untouched. They cannot be
/// read or modified, and they have no JSON representation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnknownFields(Vec<u8>);

impl UnknownFields {
    fn merge(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut impl Buf,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        let out = &mut self.0;
        match wire_type {
            WireType::Varint => {
                let mut value = 0u64;
                encoding::uint64::merge(wire_type, &mut value, buf, ctx)?;
                encoding::uint64::encode(tag, &value, out);
            }
            WireType::SixtyFourBit => {
                let mut value = 0u64;
                encoding::fixed64::merge(wire_type, &mut value, buf, ctx)?;
                encoding::fixed64::encode(tag, &value, out);
            }
            WireType::ThirtyTwoBit => {
                let mut value = 0u32;
                encoding::fixed32::merge(wire_type, &mut value, buf, ctx)?;
                encoding::fixed32::encode(tag, &value, out);
            }
            WireType::LengthDelimited => {
                let mut value = Vec::<u8>::new();
                encoding::bytes::merge(wire_type, &mut value, buf, ctx)?;
                encoding::bytes::encode(tag, &value, out);
            }
            // Groups are a proto2 feature that the proto3 function API
            // cannot use, so they are skipped as prost skips them.
            WireType::StartGroup | WireType::EndGroup => {
                encoding::skip_field(wire_type, tag, buf, ctx)?;
            }
        }
        Ok(())
    }

    fn encode(&self, buf: &mut impl BufMut) {
        buf.put_slice(&self.0);
    }

    fn encoded_len(&self) -> usize {
        self.0.len()
    }
}

fn in_field(message: &'static str, field: &'static str) -> impl FnOnce(DecodeError) -> DecodeError {
    move |mut error| {
        error.push(message, field);
        error
    }
}

/// State of the XR (XR) and any resources.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    /// The state of the XR (XR).
    pub composite: Option<Resource>,
    /// The state of any other resources. In composition functions these are the
    /// composed resources. In operation functions they're arbitrary resources that
    /// the operation wants to create or update.
    pub resources: HashMap<String, Resource>,
    /// Fields this SDK's proto does not know, kept and re-encoded.
    pub unknown_fields: UnknownFields,
}

impl Message for State {
    fn encode_raw(&self, buf: &mut impl BufMut) {
        if let Some(composite) = &self.composite {
            encoding::message::encode(1, composite, buf);
        }
        encoding::hash_map::encode(
            encoding::string::encode,
            encoding::string::encoded_len,
            encoding::message::encode,
            encoding::message::encoded_len,
            2,
            &self.resources,
            buf,
        );
        self.unknown_fields.encode(buf);
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut impl Buf,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        match tag {
            1 => encoding::message::merge(
                wire_type,
                self.composite.get_or_insert_with(Default::default),
                buf,
                ctx,
            )
            .map_err(in_field("State", "composite")),
            2 => encoding::check_wire_type(WireType::LengthDelimited, wire_type)
                .and_then(|()| {
                    encoding::hash_map::merge(
                        encoding::string::merge,
                        encoding::message::merge,
                        &mut self.resources,
                        buf,
                        ctx,
                    )
                })
                .map_err(in_field("State", "resources")),
            _ => self.unknown_fields.merge(tag, wire_type, buf, ctx),
        }
    }

    fn encoded_len(&self) -> usize {
        self.composite
            .as_ref()
            .map_or(0, |composite| encoding::message::encoded_len(1, composite))
            + encoding::hash_map::encoded_len(
                encoding::string::encoded_len,
                encoding::message::encoded_len,
                2,
                &self.resources,
            )
            + self.unknown_fields.encoded_len()
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

/// A Resource represents the state of a Kubernetes resource.
#[derive(Clone, Default, PartialEq)]
pub struct Resource {
    /// The JSON representation of the resource.
    ///
    /// * Crossplane will set this field in a RunFunctionRequest to the entire
    ///   observed state of a resource - including its metadata, spec, and status.
    ///
    /// * A function should set this field in a RunFunctionRequest to communicate
    ///   the desired state of the resource.
    ///
    /// * A function may only specify the desired status of a XR - not its metadata
    ///   or spec. A function should not return desired metadata or spec for a XR.
    ///   This will be ignored.
    ///
    /// * A function may not specify the desired status of any other resource -
    ///   e.g. composed resources. It may only specify their metadata and spec.
    ///   Status will be ignored.
    pub resource: Option<pbjson_types::Struct>,
    /// The resource's connection details.
    ///
    /// * Crossplane will set this field in a RunFunctionRequest to communicate the
    ///   the observed connection details of a composite or composed resource.
    ///
    /// * A function should set this field in a RunFunctionResponse to indicate the
    ///   desired connection details of legacy XRs. For modern XRs, this will be ignored.
    ///
    /// * A function should not set this field in a RunFunctionResponse to indicate
    ///   the desired connection details of a composed resource. This will be
    ///   ignored.
    ///
    /// Connection details are only used for composition. They're ignored by
    /// Operations.
    pub connection_details: HashMap<String, Vec<u8>>,
    /// Ready indicates whether the resource should be considered ready.
    ///
    /// * Crossplane will never set this field in a RunFunctionRequest.
    ///
    /// * A function should set this field to READY_TRUE in a RunFunctionResponse
    ///   to indicate that a desired resource is ready.
    ///
    /// * A function should set this field to READY_TRUE in a RunFunctionResponse
    ///   to indicate that a desired XR is ready. This overwrites the standard
    ///   readiness detection that determines the ready state of the composite by the
    ///   ready state of the composed resources.
    ///
    /// Ready is only used for composition. It's ignored by Operations.
    pub ready: i32,
    /// Fields this SDK's proto does not know, kept and re-encoded.
    pub unknown_fields: UnknownFields,
}

impl Resource {
    /// Returns the enum value of `ready`, or the default if the field is set
    /// to an invalid enum value.
    pub fn ready(&self) -> Ready {
        Ready::try_from(self.ready).unwrap_or_default()
    }

    /// Sets `ready` to the provided enum value.
    pub fn set_ready(&mut self, value: Ready) {
        self.ready = value as i32;
    }
}

/// Prints `ready` as prost does: by variant name, or as a number when the
/// value is not a known variant.
struct ReadyDebug(i32);

impl fmt::Debug for ReadyDebug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match Ready::try_from(self.0) {
            Ok(ready) => fmt::Debug::fmt(&ready, f),
            Err(_) => fmt::Debug::fmt(&self.0, f),
        }
    }
}

impl fmt::Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource")
            .field("resource", &self.resource)
            .field("connection_details", &self.connection_details)
            .field("ready", &ReadyDebug(self.ready))
            .field("unknown_fields", &self.unknown_fields)
            .finish()
    }
}

impl Message for Resource {
    fn encode_raw(&self, buf: &mut impl BufMut) {
        if let Some(resource) = &self.resource {
            encoding::message::encode(1, resource, buf);
        }
        encoding::hash_map::encode(
            encoding::string::encode,
            encoding::string::encoded_len,
            encoding::bytes::encode,
            encoding::bytes::encoded_len,
            2,
            &self.connection_details,
            buf,
        );
        if self.ready != Ready::default() as i32 {
            encoding::int32::encode(3, &self.ready, buf);
        }
        self.unknown_fields.encode(buf);
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut impl Buf,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        match tag {
            1 => encoding::message::merge(
                wire_type,
                self.resource.get_or_insert_with(Default::default),
                buf,
                ctx,
            )
            .map_err(in_field("Resource", "resource")),
            2 => encoding::check_wire_type(WireType::LengthDelimited, wire_type)
                .and_then(|()| {
                    encoding::hash_map::merge(
                        encoding::string::merge,
                        encoding::bytes::merge,
                        &mut self.connection_details,
                        buf,
                        ctx,
                    )
                })
                .map_err(in_field("Resource", "connection_details")),
            3 => encoding::int32::merge(wire_type, &mut self.ready, buf, ctx)
                .map_err(in_field("Resource", "ready")),
            _ => self.unknown_fields.merge(tag, wire_type, buf, ctx),
        }
    }

    fn encoded_len(&self) -> usize {
        let ready = if self.ready != Ready::default() as i32 {
            encoding::int32::encoded_len(3, &self.ready)
        } else {
            0
        };
        self.resource
            .as_ref()
            .map_or(0, |resource| encoding::message::encoded_len(1, resource))
            + encoding::hash_map::encoded_len(
                encoding::string::encoded_len,
                encoding::bytes::encoded_len,
                2,
                &self.connection_details,
            )
            + ready
            + self.unknown_fields.encoded_len()
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}
