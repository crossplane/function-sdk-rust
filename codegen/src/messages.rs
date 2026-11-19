//! Rewrites the messages prost-build generates so that they keep unknown
//! fields.
//!
//! prost's `Message` derive skips a field it does not know, and nothing in
//! prost-build changes that (tokio-rs/prost#2). So prost-build still writes
//! every struct, with its names, types, docs and derives, and this pass
//! replaces the derive: each message gets an `unknown_fields` field and the
//! impls the derive would have produced, except that the fallback arm of
//! `merge_field` keeps the field's bytes and `encode_raw` writes them back.
//!
//! The pass reads the `#[prost(...)]` attribute of every field and knows the
//! kinds the function proto uses. A kind it does not know is an error naming
//! the field, never a guess: add the kind here and let
//! `sdk/tests/generated_messages.rs` hold it to prost's own derive.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, Fields, Ident, Item, ItemStruct, Lit, Meta, Path, Token};

/// The messages of a generated file, rewritten, and their names as the
/// generated serde code spells them.
pub struct Rewritten {
    pub source: String,
    pub messages: Vec<String>,
}

/// Rewrites every struct that derives `::prost::Message` in `prost`, the
/// part of prost-build's output that holds the messages and enumerations.
pub fn rewrite(prost: &str) -> Result<Rewritten, String> {
    let mut file =
        syn::parse_file(prost).map_err(|e| format!("cannot parse prost's output: {e}"))?;
    let mut messages = Vec::new();
    file.items = rewrite_items(std::mem::take(&mut file.items), "", &mut messages)?;
    Ok(Rewritten {
        source: prettyplease::unparse(&file),
        messages,
    })
}

fn rewrite_items(
    items: Vec<Item>,
    module: &str,
    messages: &mut Vec<String>,
) -> Result<Vec<Item>, String> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Item::Struct(mut message) if derives_message(&message) => {
                let impls = rewrite_message(&mut message)?;
                messages.push(format!("{module}{}", message.ident));
                out.push(Item::Struct(message));
                let impls: syn::File = syn::parse2(impls)
                    .map_err(|e| format!("the impls written for a message do not parse: {e}"))?;
                out.extend(impls.items);
            }
            Item::Mod(mut nested) => {
                if let Some((brace, items)) = nested.content.take() {
                    let path = format!("{module}{}::", nested.ident);
                    nested.content = Some((brace, rewrite_items(items, &path, messages)?));
                }
                out.push(Item::Mod(nested));
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

fn derives_message(item: &ItemStruct) -> bool {
    derives(item).iter().any(is_prost_message)
}

fn derives(item: &ItemStruct) -> Vec<Path> {
    item.attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .filter_map(|attr| {
            attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
                .ok()
        })
        .flatten()
        .collect()
}

fn is_prost_message(path: &Path) -> bool {
    let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    segments == ["prost", "Message"]
}

/// How often a field occurs.
#[derive(Clone, Copy, PartialEq)]
enum Label {
    Plain,
    Optional,
    Repeated,
}

/// The value of a map with string keys.
#[derive(Clone, Copy)]
enum MapValue {
    String,
    Bytes,
    Message,
}

/// The field kinds this pass writes code for.
enum Kind {
    String(Label),
    /// The field is an `i32`, with the enumeration it holds.
    Enumeration(Path, Label),
    Message(Label),
    Map(MapValue),
    /// The field is an `Option` of the enum prost derives `Oneof` for.
    Oneof(Path),
}

struct Field {
    ident: Ident,
    kind: Kind,
    tags: Vec<u32>,
}

/// Replaces the `Message` derive of one struct: strips what belongs to the
/// derive, adds `unknown_fields`, and returns the impls to put after it.
fn rewrite_message(message: &mut ItemStruct) -> Result<TokenStream, String> {
    let name = message.ident.clone();
    let Fields::Named(named) = &mut message.fields else {
        return Err(format!("{name}: a message is a struct with named fields"));
    };

    let mut fields = Vec::new();
    for field in &mut named.named {
        let ident = field.ident.clone().expect("a named field");
        let position = field
            .attrs
            .iter()
            .position(|attr| attr.path().is_ident("prost"));
        let Some(position) = position else {
            return Err(format!("{name}.{ident}: no #[prost(...)] attribute"));
        };
        let attr = field.attrs.remove(position);
        fields.push(read_field(&name, ident, &attr)?);
    }
    named.named.push(syn::parse_quote! {
        /// Fields this SDK's proto does not know, kept and re-encoded.
        pub unknown_fields: crate::proto::UnknownFields
    });

    // The derive gave the message Default and Debug. Default is derived
    // (every field of a proto3 message defaults to its type's default) and
    // Debug is written below. A message with unknown fields owns a buffer,
    // so it cannot be Copy.
    let kept: Vec<Path> = derives(message)
        .into_iter()
        .filter(|path| !is_prost_message(path) && !path.is_ident("Copy"))
        .collect();
    message.attrs.retain(|attr| !attr.path().is_ident("derive"));
    message
        .attrs
        .push(syn::parse_quote!(#[derive(#(#kept,)* Default)]));

    // Debug prints the fields in declaration order; the wire has them in
    // tag order, a oneof at its lowest tag.
    let debug: Vec<TokenStream> = fields.iter().map(Field::debug).collect();
    let methods: Vec<TokenStream> = fields.iter().filter_map(Field::methods).collect();
    fields.sort_by_key(|field| field.tags.iter().copied().min().unwrap_or(0));
    let encode = fields.iter().map(Field::encode);
    let merge = fields.iter().map(Field::merge);
    let encoded_len = fields.iter().map(Field::encoded_len);

    let methods = if methods.is_empty() {
        quote!()
    } else {
        quote! {
            impl #name {
                #(#methods)*
            }
        }
    };

    Ok(quote! {
        impl ::prost::Message for #name {
            fn encode_raw(&self, buf: &mut impl ::prost::bytes::BufMut) {
                #(#encode)*
                self.unknown_fields.encode(buf);
            }

            fn merge_field(
                &mut self,
                tag: u32,
                wire_type: ::prost::encoding::WireType,
                buf: &mut impl ::prost::bytes::Buf,
                ctx: ::prost::encoding::DecodeContext,
            ) -> ::core::result::Result<(), ::prost::DecodeError> {
                const STRUCT_NAME: &str = stringify!(#name);
                match tag {
                    #(#merge)*
                    _ => self.unknown_fields.merge(tag, wire_type, buf, ctx),
                }
            }

            #[inline]
            fn encoded_len(&self) -> usize {
                0 #(+ #encoded_len)* + self.unknown_fields.encoded_len()
            }

            fn clear(&mut self) {
                *self = Self::default();
            }
        }

        impl ::core::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!(#name))
                    #(#debug)*
                    .field("unknown_fields", &self.unknown_fields)
                    .finish()
            }
        }

        #methods
    })
}

/// Reads one `#[prost(...)]` attribute as prost-build writes it.
fn read_field(message: &Ident, ident: Ident, attr: &Attribute) -> Result<Field, String> {
    let unknown = || {
        format!(
            "{message}.{ident}: codegen does not write the field kind `{}` yet; add it to codegen/src/messages.rs",
            quote!(#attr)
        )
    };
    let metas = attr
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .map_err(|_| unknown())?;

    let (mut ty, mut label, mut tags) = (None, Label::Plain, Vec::new());
    for meta in &metas {
        let word = meta
            .path()
            .get_ident()
            .map(Ident::to_string)
            .ok_or_else(unknown)?;
        match (word.as_str(), meta) {
            ("optional", Meta::Path(_)) => label = Label::Optional,
            ("repeated", Meta::Path(_)) => label = Label::Repeated,
            ("string" | "message", Meta::Path(_)) => ty = Some((word, String::new())),
            ("enumeration" | "map" | "oneof", Meta::NameValue(_)) => {
                ty = Some((word, string_of(meta).ok_or_else(unknown)?));
            }
            ("tag" | "tags", Meta::NameValue(_)) => {
                for tag in string_of(meta).ok_or_else(unknown)?.split(',') {
                    tags.push(tag.trim().parse::<u32>().map_err(|_| unknown())?);
                }
            }
            _ => return Err(unknown()),
        }
    }
    if tags.is_empty() {
        return Err(unknown());
    }

    let path = |s: &str| syn::parse_str::<Path>(s).map_err(|_| unknown());
    let kind = match (
        ty.as_ref()
            .map(|(word, value)| (word.as_str(), value.as_str())),
        label,
    ) {
        (Some(("string", _)), Label::Plain | Label::Optional) => Kind::String(label),
        (Some(("enumeration", of)), _) => Kind::Enumeration(path(of)?, label),
        (Some(("message", _)), Label::Optional | Label::Repeated) => Kind::Message(label),
        (Some(("map", "string, string")), Label::Plain) => Kind::Map(MapValue::String),
        (Some(("map", "string, bytes")), Label::Plain) => Kind::Map(MapValue::Bytes),
        (Some(("map", "string, message")), Label::Plain) => Kind::Map(MapValue::Message),
        (Some(("oneof", of)), Label::Plain) => Kind::Oneof(path(of)?),
        _ => return Err(unknown()),
    };
    Ok(Field { ident, kind, tags })
}

fn string_of(meta: &Meta) -> Option<String> {
    let Meta::NameValue(pair) = meta else {
        return None;
    };
    let Expr::Lit(literal) = &pair.value else {
        return None;
    };
    let Lit::Str(value) = &literal.lit else {
        return None;
    };
    Some(value.value())
}

impl Field {
    /// The tag of a field that is no oneof.
    fn tag(&self) -> u32 {
        self.tags[0]
    }

    /// The statement that encodes the field, as prost's derive writes it.
    fn encode(&self) -> TokenStream {
        let (field, tag) = (&self.ident, self.tag());
        match &self.kind {
            Kind::String(Label::Plain) => quote! {
                if self.#field != "" {
                    ::prost::encoding::string::encode(#tag, &self.#field, buf);
                }
            },
            Kind::String(_) => quote! {
                if let ::core::option::Option::Some(ref value) = self.#field {
                    ::prost::encoding::string::encode(#tag, value, buf);
                }
            },
            Kind::Enumeration(of, Label::Plain) => quote! {
                if self.#field != #of::default() as i32 {
                    ::prost::encoding::int32::encode(#tag, &self.#field, buf);
                }
            },
            Kind::Enumeration(_, Label::Optional) => quote! {
                if let ::core::option::Option::Some(ref value) = self.#field {
                    ::prost::encoding::int32::encode(#tag, value, buf);
                }
            },
            // proto3 packs a repeated enumeration.
            Kind::Enumeration(_, Label::Repeated) => quote! {
                ::prost::encoding::int32::encode_packed(#tag, &self.#field, buf);
            },
            Kind::Message(Label::Repeated) => quote! {
                for msg in &self.#field {
                    ::prost::encoding::message::encode(#tag, msg, buf);
                }
            },
            Kind::Message(_) => quote! {
                if let Some(ref msg) = self.#field {
                    ::prost::encoding::message::encode(#tag, msg, buf);
                }
            },
            Kind::Map(value) => {
                let value = value.module();
                quote! {
                    ::prost::encoding::hash_map::encode(
                        ::prost::encoding::string::encode,
                        ::prost::encoding::string::encoded_len,
                        ::prost::encoding::#value::encode,
                        ::prost::encoding::#value::encoded_len,
                        #tag,
                        &self.#field,
                        buf,
                    );
                }
            }
            Kind::Oneof(_) => quote! {
                if let Some(ref oneof) = self.#field {
                    oneof.encode(buf)
                }
            },
        }
    }

    /// The arm of `merge_field` that decodes the field. An error names the
    /// message and the field, as prost's derive has it.
    fn merge(&self) -> TokenStream {
        let (field, tags) = (&self.ident, &self.tags);
        let merge = match &self.kind {
            Kind::String(Label::Plain) => quote! {
                ::prost::encoding::string::merge(wire_type, value, buf, ctx)
            },
            Kind::String(_) => quote! {
                ::prost::encoding::string::merge(
                    wire_type,
                    value.get_or_insert_with(::core::default::Default::default),
                    buf,
                    ctx,
                )
            },
            Kind::Enumeration(_, Label::Plain) => quote! {
                ::prost::encoding::int32::merge(wire_type, value, buf, ctx)
            },
            Kind::Enumeration(_, Label::Optional) => quote! {
                ::prost::encoding::int32::merge(
                    wire_type,
                    value.get_or_insert_with(::core::default::Default::default),
                    buf,
                    ctx,
                )
            },
            Kind::Enumeration(_, Label::Repeated) => quote! {
                ::prost::encoding::int32::merge_repeated(wire_type, value, buf, ctx)
            },
            Kind::Message(Label::Repeated) => quote! {
                ::prost::encoding::message::merge_repeated(wire_type, value, buf, ctx)
            },
            Kind::Message(_) => quote! {
                ::prost::encoding::message::merge(
                    wire_type,
                    value.get_or_insert_with(::core::default::Default::default),
                    buf,
                    ctx,
                )
            },
            // The wire type is checked here: hash_map::merge does not.
            Kind::Map(value) => {
                let value = value.module();
                quote! {
                    ::prost::encoding::check_wire_type(
                        ::prost::encoding::WireType::LengthDelimited,
                        wire_type,
                    )
                    .and_then(|()| {
                        ::prost::encoding::hash_map::merge(
                            ::prost::encoding::string::merge,
                            ::prost::encoding::#value::merge,
                            value,
                            buf,
                            ctx,
                        )
                    })
                }
            }
            Kind::Oneof(of) => quote! {
                #of::merge(value, tag, wire_type, buf, ctx)
            },
        };
        quote! {
            #(#tags)|* => {
                let value = &mut self.#field;
                #merge.map_err(|mut error| {
                    error.push(STRUCT_NAME, stringify!(#field));
                    error
                })
            }
        }
    }

    /// The expression that is the field's encoded length.
    fn encoded_len(&self) -> TokenStream {
        let (field, tag) = (&self.ident, self.tag());
        match &self.kind {
            Kind::String(Label::Plain) => quote! {
                if self.#field != "" {
                    ::prost::encoding::string::encoded_len(#tag, &self.#field)
                } else {
                    0
                }
            },
            Kind::String(_) => quote! {
                self.#field
                    .as_ref()
                    .map_or(0, |value| ::prost::encoding::string::encoded_len(#tag, value))
            },
            Kind::Enumeration(of, Label::Plain) => quote! {
                if self.#field != #of::default() as i32 {
                    ::prost::encoding::int32::encoded_len(#tag, &self.#field)
                } else {
                    0
                }
            },
            Kind::Enumeration(_, Label::Optional) => quote! {
                self.#field
                    .as_ref()
                    .map_or(0, |value| ::prost::encoding::int32::encoded_len(#tag, value))
            },
            Kind::Enumeration(_, Label::Repeated) => quote! {
                ::prost::encoding::int32::encoded_len_packed(#tag, &self.#field)
            },
            Kind::Message(Label::Repeated) => quote! {
                ::prost::encoding::message::encoded_len_repeated(#tag, &self.#field)
            },
            Kind::Message(_) => quote! {
                self.#field
                    .as_ref()
                    .map_or(0, |msg| ::prost::encoding::message::encoded_len(#tag, msg))
            },
            Kind::Map(value) => {
                let value = value.module();
                quote! {
                    ::prost::encoding::hash_map::encoded_len(
                        ::prost::encoding::string::encoded_len,
                        ::prost::encoding::#value::encoded_len,
                        #tag,
                        &self.#field,
                    )
                }
            }
            Kind::Oneof(of) => quote! {
                self.#field.as_ref().map_or(0, #of::encoded_len)
            },
        }
    }

    /// The `.field(..)` call of `Debug`. An enumeration prints by variant
    /// name, as prost's derive prints it.
    fn debug(&self) -> TokenStream {
        let field = &self.ident;
        let by_name = quote!(crate::proto::unknown::enum_debug);
        match &self.kind {
            Kind::Enumeration(of, Label::Plain) => quote! {
                .field(stringify!(#field), &#by_name::<#of>(&self.#field))
            },
            Kind::Enumeration(of, Label::Optional) => quote! {
                .field(stringify!(#field), &self.#field.as_ref().map(#by_name::<#of>))
            },
            Kind::Enumeration(of, Label::Repeated) => quote! {
                .field(
                    stringify!(#field),
                    &self.#field.iter().map(#by_name::<#of>).collect::<::prost::alloc::vec::Vec<_>>(),
                )
            },
            _ => quote!(.field(stringify!(#field), &self.#field)),
        }
    }

    /// The accessors prost's derive adds for an enumeration and for an
    /// optional string, under the names and docs it gives them.
    fn methods(&self) -> Option<TokenStream> {
        let field = &self.ident;
        let name = field.to_string();
        let name = name.strip_prefix("r#").unwrap_or(&name);
        let set = format_ident!("set_{name}");
        let set_doc = format!("Sets `{name}` to the provided enum value.");
        Some(match &self.kind {
            Kind::Enumeration(of, Label::Plain) => {
                let get_doc = format!(
                    "Returns the enum value of `{name}`, or the default if the field is set to an invalid enum value."
                );
                quote! {
                    #[doc = #get_doc]
                    pub fn #field(&self) -> #of {
                        ::core::convert::TryFrom::try_from(self.#field).unwrap_or(#of::default())
                    }

                    #[doc = #set_doc]
                    pub fn #set(&mut self, value: #of) {
                        self.#field = value as i32;
                    }
                }
            }
            Kind::Enumeration(of, Label::Optional) => {
                let get_doc = format!(
                    "Returns the enum value of `{name}`, or the default if the field is unset or set to an invalid enum value."
                );
                quote! {
                    #[doc = #get_doc]
                    pub fn #field(&self) -> #of {
                        self.#field
                            .and_then(|x| {
                                let result: ::core::result::Result<#of, _> =
                                    ::core::convert::TryFrom::try_from(x);
                                result.ok()
                            })
                            .unwrap_or(#of::default())
                    }

                    #[doc = #set_doc]
                    pub fn #set(&mut self, value: #of) {
                        self.#field = ::core::option::Option::Some(value as i32);
                    }
                }
            }
            Kind::Enumeration(of, Label::Repeated) => {
                let push = format_ident!("push_{name}");
                let iter_doc = format!(
                    "Returns an iterator which yields the valid enum values contained in `{name}`."
                );
                let push_doc = format!("Appends the provided enum value to `{name}`.");
                quote! {
                    #[doc = #iter_doc]
                    pub fn #field(
                        &self,
                    ) -> ::core::iter::FilterMap<
                        ::core::iter::Cloned<::core::slice::Iter<'_, i32>>,
                        fn(i32) -> ::core::option::Option<#of>,
                    > {
                        self.#field.iter().cloned().filter_map(|x| {
                            let result: ::core::result::Result<#of, _> =
                                ::core::convert::TryFrom::try_from(x);
                            result.ok()
                        })
                    }

                    #[doc = #push_doc]
                    pub fn #push(&mut self, value: #of) {
                        self.#field.push(value as i32);
                    }
                }
            }
            Kind::String(Label::Optional) => {
                let get_doc = format!(
                    "Returns the value of `{name}`, or the default value if `{name}` is unset."
                );
                quote! {
                    #[doc = #get_doc]
                    pub fn #field(&self) -> &str {
                        match self.#field {
                            ::core::option::Option::Some(ref val) => &val[..],
                            ::core::option::Option::None => "",
                        }
                    }
                }
            }
            _ => return None,
        })
    }
}

impl MapValue {
    /// The module of `prost::encoding` that encodes the value.
    fn module(self) -> Ident {
        match self {
            MapValue::String => format_ident!("string"),
            MapValue::Bytes => format_ident!("bytes"),
            MapValue::Message => format_ident!("message"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A message whose fields are declared out of tag order, with a oneof
    /// that lists its highest tag first. On the wire the order is `tag` (1),
    /// `source` (3), `kind` (4): the declaration order and a sort by a
    /// oneof's first tag are both another order.
    const PROST: &str = r#"
/// A message.
#[allow(clippy::all)]
#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]
pub struct Meta {
    #[prost(enumeration = "Kind", optional, tag = "4")]
    pub kind: ::core::option::Option<i32>,
    #[prost(oneof = "meta::Source", tags = "6, 3")]
    pub source: ::core::option::Option<meta::Source>,
    /// Its tag.
    #[prost(string, tag = "1")]
    pub tag: ::prost::alloc::string::String,
}
/// Nested message and enum types in `Meta`.
pub mod meta {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Source {
        #[prost(string, tag = "3")]
        Name(::prost::alloc::string::String),
        #[prost(message, tag = "6")]
        Inner(Inner),
    }
    #[derive(Clone, Copy, PartialEq, ::prost::Message)]
    pub struct Inner {}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Kind {
    Unspecified = 0,
}
"#;

    #[test]
    fn a_message_keeps_what_prost_wrote_but_the_derive() {
        let got = rewrite(PROST).expect("rewrites");
        assert_eq!(got.messages, ["Meta", "meta::Inner"]);
        let source = got.source;

        // The struct: its docs and other derives stay, the prost attributes
        // go, and the field is added.
        assert!(source.contains("/// A message.\n#[allow(clippy::all)]\n#[derive(Clone, PartialEq, Eq, Hash, Default)]\npub struct Meta {"), "{source}");
        assert!(
            source.contains("    /// Its tag.\n    pub tag: ::prost::alloc::string::String,\n"),
            "{source}"
        );
        assert!(source.contains("    /// Fields this SDK's proto does not know, kept and re-encoded.\n    pub unknown_fields: crate::proto::UnknownFields,\n"), "{source}");
        assert!(!source.contains("::prost::Message)]"), "{source}");
        assert!(
            !source.contains("#[prost(string, tag = \"1\")]\n    pub tag"),
            "{source}"
        );

        // The impls: the fallback arm keeps the field, a oneof takes every
        // tag it has, the accessors are back.
        assert!(
            source.contains("impl ::prost::Message for Meta {"),
            "{source}"
        );
        assert!(
            source.contains("_ => self.unknown_fields.merge(tag, wire_type, buf, ctx),"),
            "{source}"
        );
        assert!(source.contains("6u32 | 3u32 => {"), "{source}");
        assert!(
            source.contains("pub fn set_kind(&mut self, value: Kind) {"),
            "{source}"
        );

        // What is no message stays as prost wrote it, a nested message
        // loses Copy, and the result is a file rustc can read.
        assert!(
            source.contains("#[derive(Clone, PartialEq, ::prost::Oneof)]\n    pub enum Source {"),
            "{source}"
        );
        assert!(
            source.contains("        #[prost(string, tag = \"3\")]\n        Name("),
            "{source}"
        );
        assert!(
            source.contains("    #[derive(Clone, PartialEq, Default)]\n    pub struct Inner {"),
            "{source}"
        );
        assert!(
            source.contains("::prost::Enumeration)]\n#[repr(i32)]\npub enum Kind {"),
            "{source}"
        );
        syn::parse_file(&source).expect("the rewritten file parses");
    }

    #[test]
    fn fields_are_encoded_in_tag_order_with_a_oneof_at_its_lowest_tag() {
        let source = rewrite(PROST).expect("rewrites").source;
        let encode = &source[source.find("fn encode_raw").expect("encode_raw")..];
        let tag = encode.find("encode(1u32").expect("tag 1");
        let oneof = encode.find("oneof.encode(buf)").expect("the oneof");
        let kind = encode.find("encode(4u32").expect("tag 4");
        let unknown = encode
            .find("self.unknown_fields.encode(buf)")
            .expect("the unknown fields");
        assert!(tag < oneof && oneof < kind && kind < unknown, "{encode}");
    }

    #[test]
    fn debug_prints_the_fields_as_declared() {
        let source = rewrite(PROST).expect("rewrites").source;
        let debug = &source[source.find("fn fmt").expect("the Debug impl")..];
        let kind = debug.find("stringify!(kind)").expect("kind");
        let oneof = debug.find("stringify!(source)").expect("source");
        let tag = debug.find("stringify!(tag)").expect("tag");
        let unknown = debug
            .find("\"unknown_fields\"")
            .expect("the unknown fields");
        assert!(kind < oneof && oneof < tag && tag < unknown, "{debug}");
    }

    #[test]
    fn a_field_kind_the_pass_does_not_know_is_an_error_naming_it() {
        let prost = r#"
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Meta {
    #[prost(bool, tag = "1")]
    pub ready: bool,
}
"#;
        let error = rewrite(prost).err().expect("refused");
        assert_eq!(
            error,
            "Meta.ready: codegen does not write the field kind `# [prost (bool , tag = \"1\")]` yet; add it to codegen/src/messages.rs"
        );
    }
}
