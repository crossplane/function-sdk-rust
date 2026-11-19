//! What the generated messages share so that they keep unknown fields.
//!
//! prost discards fields it does not know. Codegen writes the
//! `prost::Message` impl of every message itself (`codegen/src/messages.rs`)
//! so that fields a newer Crossplane adds survive a function built against
//! an older proto, as they do in function-sdk-go. Known fields are decoded
//! and encoded as prost generates it.

use std::fmt;
use std::marker::PhantomData;

use prost::DecodeError;
use prost::bytes::{Buf, BufMut};
use prost::encoding::{self, DecodeContext, WireType};

/// Fields of a message that this SDK's proto does not know.
///
/// Crossplane may add fields to a message after this SDK was built. They
/// are kept when a message is decoded and written back when it is encoded,
/// so a function passes them down the pipeline untouched. They cannot be
/// read or modified, and they have no JSON representation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct UnknownFields(Vec<u8>);

impl UnknownFields {
    pub(crate) fn merge(
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

    pub(crate) fn encode(&self, buf: &mut impl BufMut) {
        buf.put_slice(&self.0);
    }

    pub(crate) fn encoded_len(&self) -> usize {
        self.0.len()
    }
}

/// Prints an enumeration field as prost does: by variant name, or as the
/// number when the value is not a known variant.
pub(crate) struct EnumDebug<'a, E>(&'a i32, PhantomData<E>);

pub(crate) fn enum_debug<E>(value: &i32) -> EnumDebug<'_, E> {
    EnumDebug(value, PhantomData)
}

impl<E: TryFrom<i32> + fmt::Debug> fmt::Debug for EnumDebug<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match E::try_from(*self.0) {
            Ok(variant) => fmt::Debug::fmt(&variant, f),
            Err(_) => fmt::Debug::fmt(self.0, f),
        }
    }
}
