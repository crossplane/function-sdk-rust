impl serde::Serialize for Capability {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "CAPABILITY_UNSPECIFIED",
            Self::Capabilities => "CAPABILITY_CAPABILITIES",
            Self::RequiredResources => "CAPABILITY_REQUIRED_RESOURCES",
            Self::Credentials => "CAPABILITY_CREDENTIALS",
            Self::Conditions => "CAPABILITY_CONDITIONS",
            Self::RequiredSchemas => "CAPABILITY_REQUIRED_SCHEMAS",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Capability {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "CAPABILITY_UNSPECIFIED",
            "CAPABILITY_CAPABILITIES",
            "CAPABILITY_REQUIRED_RESOURCES",
            "CAPABILITY_CREDENTIALS",
            "CAPABILITY_CONDITIONS",
            "CAPABILITY_REQUIRED_SCHEMAS",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = Capability;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "CAPABILITY_UNSPECIFIED" => Ok(Capability::Unspecified),
                    "CAPABILITY_CAPABILITIES" => Ok(Capability::Capabilities),
                    "CAPABILITY_REQUIRED_RESOURCES" => Ok(Capability::RequiredResources),
                    "CAPABILITY_CREDENTIALS" => Ok(Capability::Credentials),
                    "CAPABILITY_CONDITIONS" => Ok(Capability::Conditions),
                    "CAPABILITY_REQUIRED_SCHEMAS" => Ok(Capability::RequiredSchemas),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Condition {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.r#type.is_empty() {
            len += 1;
        }
        if self.status != 0 {
            len += 1;
        }
        if !self.reason.is_empty() {
            len += 1;
        }
        if self.message.is_some() {
            len += 1;
        }
        if self.target.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.Condition", len)?;
        if !self.r#type.is_empty() {
            struct_ser.serialize_field("type", &self.r#type)?;
        }
        if self.status != 0 {
            let v = Status::try_from(self.status)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.status)))?;
            struct_ser.serialize_field("status", &v)?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        if let Some(v) = self.message.as_ref() {
            struct_ser.serialize_field("message", v)?;
        }
        if let Some(v) = self.target.as_ref() {
            let v = Target::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("target", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Condition {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "status",
            "reason",
            "message",
            "target",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Status,
            Reason,
            Message,
            Target,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "type" => Ok(GeneratedField::Type),
                            "status" => Ok(GeneratedField::Status),
                            "reason" => Ok(GeneratedField::Reason),
                            "message" => Ok(GeneratedField::Message),
                            "target" => Ok(GeneratedField::Target),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Condition;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.Condition")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Condition, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut status__ = None;
                let mut reason__ = None;
                let mut message__ = None;
                let mut target__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map_.next_value::<Status>()? as i32);
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = map_.next_value()?;
                        }
                        GeneratedField::Target => {
                            if target__.is_some() {
                                return Err(serde::de::Error::duplicate_field("target"));
                            }
                            target__ = map_.next_value::<::std::option::Option<Target>>()?.map(|x| x as i32);
                        }
                    }
                }
                Ok(Condition {
                    r#type: r#type__.unwrap_or_default(),
                    status: status__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                    message: message__,
                    target: target__,
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.Condition", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CredentialData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.CredentialData", len)?;
        if !self.data.is_empty() {
            let v: std::collections::HashMap<_, _> = self.data.iter()
                .map(|(k, v)| (k, pbjson::private::base64::encode(v))).collect();
            struct_ser.serialize_field("data", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CredentialData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Data,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "data" => Ok(GeneratedField::Data),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CredentialData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.CredentialData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CredentialData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = Some(
                                map_.next_value::<std::collections::HashMap<_, ::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|(k,v)| (k, v.0)).collect()
                            );
                        }
                    }
                }
                Ok(CredentialData {
                    data: data__.unwrap_or_default(),
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.CredentialData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Credentials {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.source.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.Credentials", len)?;
        if let Some(v) = self.source.as_ref() {
            match v {
                credentials::Source::CredentialData(v) => {
                    struct_ser.serialize_field("credentialData", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Credentials {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "credential_data",
            "credentialData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CredentialData,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "credentialData" | "credential_data" => Ok(GeneratedField::CredentialData),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Credentials;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.Credentials")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Credentials, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut source__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CredentialData => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credentialData"));
                            }
                            source__ = map_.next_value::<::std::option::Option<_>>()?.map(credentials::Source::CredentialData)
;
                        }
                    }
                }
                Ok(Credentials {
                    source: source__,
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.Credentials", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MatchLabels {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.labels.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.MatchLabels", len)?;
        if !self.labels.is_empty() {
            struct_ser.serialize_field("labels", &self.labels)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MatchLabels {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "labels",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Labels,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "labels" => Ok(GeneratedField::Labels),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MatchLabels;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.MatchLabels")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MatchLabels, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut labels__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Labels => {
                            if labels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("labels"));
                            }
                            labels__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(MatchLabels {
                    labels: labels__.unwrap_or_default(),
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.MatchLabels", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Ready {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "READY_UNSPECIFIED",
            Self::True => "READY_TRUE",
            Self::False => "READY_FALSE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Ready {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "READY_UNSPECIFIED",
            "READY_TRUE",
            "READY_FALSE",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = Ready;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "READY_UNSPECIFIED" => Ok(Ready::Unspecified),
                    "READY_TRUE" => Ok(Ready::True),
                    "READY_FALSE" => Ok(Ready::False),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for RequestMeta {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tag.is_empty() {
            len += 1;
        }
        if !self.capabilities.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.RequestMeta", len)?;
        if !self.tag.is_empty() {
            struct_ser.serialize_field("tag", &self.tag)?;
        }
        if !self.capabilities.is_empty() {
            let v = self.capabilities.iter().cloned().map(|v| {
                Capability::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("capabilities", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RequestMeta {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag",
            "capabilities",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tag,
            Capabilities,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tag" => Ok(GeneratedField::Tag),
                            "capabilities" => Ok(GeneratedField::Capabilities),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RequestMeta;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.RequestMeta")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RequestMeta, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag__ = None;
                let mut capabilities__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Tag => {
                            if tag__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tag"));
                            }
                            tag__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Capabilities => {
                            if capabilities__.is_some() {
                                return Err(serde::de::Error::duplicate_field("capabilities"));
                            }
                            capabilities__ = Some(map_.next_value::<Vec<Capability>>()?.into_iter().map(|x| x as i32).collect());
                        }
                    }
                }
                Ok(RequestMeta {
                    tag: tag__.unwrap_or_default(),
                    capabilities: capabilities__.unwrap_or_default(),
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.RequestMeta", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Requirements {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.extra_resources.is_empty() {
            len += 1;
        }
        if !self.resources.is_empty() {
            len += 1;
        }
        if !self.schemas.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.Requirements", len)?;
        if !self.extra_resources.is_empty() {
            struct_ser.serialize_field("extraResources", &self.extra_resources)?;
        }
        if !self.resources.is_empty() {
            struct_ser.serialize_field("resources", &self.resources)?;
        }
        if !self.schemas.is_empty() {
            struct_ser.serialize_field("schemas", &self.schemas)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Requirements {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "extra_resources",
            "extraResources",
            "resources",
            "schemas",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExtraResources,
            Resources,
            Schemas,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "extraResources" | "extra_resources" => Ok(GeneratedField::ExtraResources),
                            "resources" => Ok(GeneratedField::Resources),
                            "schemas" => Ok(GeneratedField::Schemas),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Requirements;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.Requirements")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Requirements, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut extra_resources__ = None;
                let mut resources__ = None;
                let mut schemas__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ExtraResources => {
                            if extra_resources__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extraResources"));
                            }
                            extra_resources__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Resources => {
                            if resources__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resources"));
                            }
                            resources__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Schemas => {
                            if schemas__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemas"));
                            }
                            schemas__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(Requirements {
                    extra_resources: extra_resources__.unwrap_or_default(),
                    resources: resources__.unwrap_or_default(),
                    schemas: schemas__.unwrap_or_default(),
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.Requirements", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Resource {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.resource.is_some() {
            len += 1;
        }
        if !self.connection_details.is_empty() {
            len += 1;
        }
        if self.ready != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.Resource", len)?;
        if let Some(v) = self.resource.as_ref() {
            struct_ser.serialize_field("resource", v)?;
        }
        if !self.connection_details.is_empty() {
            let v: std::collections::HashMap<_, _> = self.connection_details.iter()
                .map(|(k, v)| (k, pbjson::private::base64::encode(v))).collect();
            struct_ser.serialize_field("connectionDetails", &v)?;
        }
        if self.ready != 0 {
            let v = Ready::try_from(self.ready)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.ready)))?;
            struct_ser.serialize_field("ready", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Resource {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "resource",
            "connection_details",
            "connectionDetails",
            "ready",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Resource,
            ConnectionDetails,
            Ready,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "resource" => Ok(GeneratedField::Resource),
                            "connectionDetails" | "connection_details" => Ok(GeneratedField::ConnectionDetails),
                            "ready" => Ok(GeneratedField::Ready),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Resource;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.Resource")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Resource, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut resource__ = None;
                let mut connection_details__ = None;
                let mut ready__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Resource => {
                            if resource__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resource"));
                            }
                            resource__ = map_.next_value()?;
                        }
                        GeneratedField::ConnectionDetails => {
                            if connection_details__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionDetails"));
                            }
                            connection_details__ = Some(
                                map_.next_value::<std::collections::HashMap<_, ::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|(k,v)| (k, v.0)).collect()
                            );
                        }
                        GeneratedField::Ready => {
                            if ready__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ready"));
                            }
                            ready__ = Some(map_.next_value::<Ready>()? as i32);
                        }
                    }
                }
                Ok(Resource {
                    resource: resource__,
                    connection_details: connection_details__.unwrap_or_default(),
                    ready: ready__.unwrap_or_default(),
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.Resource", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResourceSelector {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.api_version.is_empty() {
            len += 1;
        }
        if !self.kind.is_empty() {
            len += 1;
        }
        if self.namespace.is_some() {
            len += 1;
        }
        if self.r#match.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.ResourceSelector", len)?;
        if !self.api_version.is_empty() {
            struct_ser.serialize_field("apiVersion", &self.api_version)?;
        }
        if !self.kind.is_empty() {
            struct_ser.serialize_field("kind", &self.kind)?;
        }
        if let Some(v) = self.namespace.as_ref() {
            struct_ser.serialize_field("namespace", v)?;
        }
        if let Some(v) = self.r#match.as_ref() {
            match v {
                resource_selector::Match::MatchName(v) => {
                    struct_ser.serialize_field("matchName", v)?;
                }
                resource_selector::Match::MatchLabels(v) => {
                    struct_ser.serialize_field("matchLabels", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResourceSelector {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "api_version",
            "apiVersion",
            "kind",
            "namespace",
            "match_name",
            "matchName",
            "match_labels",
            "matchLabels",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ApiVersion,
            Kind,
            Namespace,
            MatchName,
            MatchLabels,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "apiVersion" | "api_version" => Ok(GeneratedField::ApiVersion),
                            "kind" => Ok(GeneratedField::Kind),
                            "namespace" => Ok(GeneratedField::Namespace),
                            "matchName" | "match_name" => Ok(GeneratedField::MatchName),
                            "matchLabels" | "match_labels" => Ok(GeneratedField::MatchLabels),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResourceSelector;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.ResourceSelector")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ResourceSelector, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut api_version__ = None;
                let mut kind__ = None;
                let mut namespace__ = None;
                let mut r#match__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ApiVersion => {
                            if api_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiVersion"));
                            }
                            api_version__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Kind => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("kind"));
                            }
                            kind__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Namespace => {
                            if namespace__.is_some() {
                                return Err(serde::de::Error::duplicate_field("namespace"));
                            }
                            namespace__ = map_.next_value()?;
                        }
                        GeneratedField::MatchName => {
                            if r#match__.is_some() {
                                return Err(serde::de::Error::duplicate_field("matchName"));
                            }
                            r#match__ = map_.next_value::<::std::option::Option<_>>()?.map(resource_selector::Match::MatchName);
                        }
                        GeneratedField::MatchLabels => {
                            if r#match__.is_some() {
                                return Err(serde::de::Error::duplicate_field("matchLabels"));
                            }
                            r#match__ = map_.next_value::<::std::option::Option<_>>()?.map(resource_selector::Match::MatchLabels)
;
                        }
                    }
                }
                Ok(ResourceSelector {
                    api_version: api_version__.unwrap_or_default(),
                    kind: kind__.unwrap_or_default(),
                    namespace: namespace__,
                    r#match: r#match__,
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.ResourceSelector", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Resources {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.items.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.Resources", len)?;
        if !self.items.is_empty() {
            struct_ser.serialize_field("items", &self.items)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Resources {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "items",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Items,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "items" => Ok(GeneratedField::Items),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Resources;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.Resources")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Resources, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut items__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Items => {
                            if items__.is_some() {
                                return Err(serde::de::Error::duplicate_field("items"));
                            }
                            items__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Resources {
                    items: items__.unwrap_or_default(),
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.Resources", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResponseMeta {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tag.is_empty() {
            len += 1;
        }
        if self.ttl.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.ResponseMeta", len)?;
        if !self.tag.is_empty() {
            struct_ser.serialize_field("tag", &self.tag)?;
        }
        if let Some(v) = self.ttl.as_ref() {
            struct_ser.serialize_field("ttl", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ResponseMeta {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag",
            "ttl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tag,
            Ttl,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tag" => Ok(GeneratedField::Tag),
                            "ttl" => Ok(GeneratedField::Ttl),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResponseMeta;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.ResponseMeta")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ResponseMeta, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag__ = None;
                let mut ttl__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Tag => {
                            if tag__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tag"));
                            }
                            tag__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Ttl => {
                            if ttl__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ttl"));
                            }
                            ttl__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ResponseMeta {
                    tag: tag__.unwrap_or_default(),
                    ttl: ttl__,
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.ResponseMeta", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Result {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.severity != 0 {
            len += 1;
        }
        if !self.message.is_empty() {
            len += 1;
        }
        if self.reason.is_some() {
            len += 1;
        }
        if self.target.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.Result", len)?;
        if self.severity != 0 {
            let v = Severity::try_from(self.severity)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.severity)))?;
            struct_ser.serialize_field("severity", &v)?;
        }
        if !self.message.is_empty() {
            struct_ser.serialize_field("message", &self.message)?;
        }
        if let Some(v) = self.reason.as_ref() {
            struct_ser.serialize_field("reason", v)?;
        }
        if let Some(v) = self.target.as_ref() {
            let v = Target::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("target", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Result {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "severity",
            "message",
            "reason",
            "target",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Severity,
            Message,
            Reason,
            Target,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "severity" => Ok(GeneratedField::Severity),
                            "message" => Ok(GeneratedField::Message),
                            "reason" => Ok(GeneratedField::Reason),
                            "target" => Ok(GeneratedField::Target),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Result;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.Result")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Result, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut severity__ = None;
                let mut message__ = None;
                let mut reason__ = None;
                let mut target__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Severity => {
                            if severity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("severity"));
                            }
                            severity__ = Some(map_.next_value::<Severity>()? as i32);
                        }
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = map_.next_value()?;
                        }
                        GeneratedField::Target => {
                            if target__.is_some() {
                                return Err(serde::de::Error::duplicate_field("target"));
                            }
                            target__ = map_.next_value::<::std::option::Option<Target>>()?.map(|x| x as i32);
                        }
                    }
                }
                Ok(Result {
                    severity: severity__.unwrap_or_default(),
                    message: message__.unwrap_or_default(),
                    reason: reason__,
                    target: target__,
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.Result", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RunFunctionRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.meta.is_some() {
            len += 1;
        }
        if self.observed.is_some() {
            len += 1;
        }
        if self.desired.is_some() {
            len += 1;
        }
        if self.input.is_some() {
            len += 1;
        }
        if self.context.is_some() {
            len += 1;
        }
        if !self.extra_resources.is_empty() {
            len += 1;
        }
        if !self.credentials.is_empty() {
            len += 1;
        }
        if !self.required_resources.is_empty() {
            len += 1;
        }
        if !self.required_schemas.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.RunFunctionRequest", len)?;
        if let Some(v) = self.meta.as_ref() {
            struct_ser.serialize_field("meta", v)?;
        }
        if let Some(v) = self.observed.as_ref() {
            struct_ser.serialize_field("observed", v)?;
        }
        if let Some(v) = self.desired.as_ref() {
            struct_ser.serialize_field("desired", v)?;
        }
        if let Some(v) = self.input.as_ref() {
            struct_ser.serialize_field("input", v)?;
        }
        if let Some(v) = self.context.as_ref() {
            struct_ser.serialize_field("context", v)?;
        }
        if !self.extra_resources.is_empty() {
            struct_ser.serialize_field("extraResources", &self.extra_resources)?;
        }
        if !self.credentials.is_empty() {
            struct_ser.serialize_field("credentials", &self.credentials)?;
        }
        if !self.required_resources.is_empty() {
            struct_ser.serialize_field("requiredResources", &self.required_resources)?;
        }
        if !self.required_schemas.is_empty() {
            struct_ser.serialize_field("requiredSchemas", &self.required_schemas)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RunFunctionRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "meta",
            "observed",
            "desired",
            "input",
            "context",
            "extra_resources",
            "extraResources",
            "credentials",
            "required_resources",
            "requiredResources",
            "required_schemas",
            "requiredSchemas",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Meta,
            Observed,
            Desired,
            Input,
            Context,
            ExtraResources,
            Credentials,
            RequiredResources,
            RequiredSchemas,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "meta" => Ok(GeneratedField::Meta),
                            "observed" => Ok(GeneratedField::Observed),
                            "desired" => Ok(GeneratedField::Desired),
                            "input" => Ok(GeneratedField::Input),
                            "context" => Ok(GeneratedField::Context),
                            "extraResources" | "extra_resources" => Ok(GeneratedField::ExtraResources),
                            "credentials" => Ok(GeneratedField::Credentials),
                            "requiredResources" | "required_resources" => Ok(GeneratedField::RequiredResources),
                            "requiredSchemas" | "required_schemas" => Ok(GeneratedField::RequiredSchemas),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RunFunctionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.RunFunctionRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RunFunctionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut meta__ = None;
                let mut observed__ = None;
                let mut desired__ = None;
                let mut input__ = None;
                let mut context__ = None;
                let mut extra_resources__ = None;
                let mut credentials__ = None;
                let mut required_resources__ = None;
                let mut required_schemas__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Meta => {
                            if meta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("meta"));
                            }
                            meta__ = map_.next_value()?;
                        }
                        GeneratedField::Observed => {
                            if observed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("observed"));
                            }
                            observed__ = map_.next_value()?;
                        }
                        GeneratedField::Desired => {
                            if desired__.is_some() {
                                return Err(serde::de::Error::duplicate_field("desired"));
                            }
                            desired__ = map_.next_value()?;
                        }
                        GeneratedField::Input => {
                            if input__.is_some() {
                                return Err(serde::de::Error::duplicate_field("input"));
                            }
                            input__ = map_.next_value()?;
                        }
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = map_.next_value()?;
                        }
                        GeneratedField::ExtraResources => {
                            if extra_resources__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extraResources"));
                            }
                            extra_resources__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Credentials => {
                            if credentials__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credentials"));
                            }
                            credentials__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::RequiredResources => {
                            if required_resources__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requiredResources"));
                            }
                            required_resources__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::RequiredSchemas => {
                            if required_schemas__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requiredSchemas"));
                            }
                            required_schemas__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(RunFunctionRequest {
                    meta: meta__,
                    observed: observed__,
                    desired: desired__,
                    input: input__,
                    context: context__,
                    extra_resources: extra_resources__.unwrap_or_default(),
                    credentials: credentials__.unwrap_or_default(),
                    required_resources: required_resources__.unwrap_or_default(),
                    required_schemas: required_schemas__.unwrap_or_default(),
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.RunFunctionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RunFunctionResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.meta.is_some() {
            len += 1;
        }
        if self.desired.is_some() {
            len += 1;
        }
        if !self.results.is_empty() {
            len += 1;
        }
        if self.context.is_some() {
            len += 1;
        }
        if self.requirements.is_some() {
            len += 1;
        }
        if !self.conditions.is_empty() {
            len += 1;
        }
        if self.output.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.RunFunctionResponse", len)?;
        if let Some(v) = self.meta.as_ref() {
            struct_ser.serialize_field("meta", v)?;
        }
        if let Some(v) = self.desired.as_ref() {
            struct_ser.serialize_field("desired", v)?;
        }
        if !self.results.is_empty() {
            struct_ser.serialize_field("results", &self.results)?;
        }
        if let Some(v) = self.context.as_ref() {
            struct_ser.serialize_field("context", v)?;
        }
        if let Some(v) = self.requirements.as_ref() {
            struct_ser.serialize_field("requirements", v)?;
        }
        if !self.conditions.is_empty() {
            struct_ser.serialize_field("conditions", &self.conditions)?;
        }
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RunFunctionResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "meta",
            "desired",
            "results",
            "context",
            "requirements",
            "conditions",
            "output",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Meta,
            Desired,
            Results,
            Context,
            Requirements,
            Conditions,
            Output,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "meta" => Ok(GeneratedField::Meta),
                            "desired" => Ok(GeneratedField::Desired),
                            "results" => Ok(GeneratedField::Results),
                            "context" => Ok(GeneratedField::Context),
                            "requirements" => Ok(GeneratedField::Requirements),
                            "conditions" => Ok(GeneratedField::Conditions),
                            "output" => Ok(GeneratedField::Output),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RunFunctionResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.RunFunctionResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RunFunctionResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut meta__ = None;
                let mut desired__ = None;
                let mut results__ = None;
                let mut context__ = None;
                let mut requirements__ = None;
                let mut conditions__ = None;
                let mut output__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Meta => {
                            if meta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("meta"));
                            }
                            meta__ = map_.next_value()?;
                        }
                        GeneratedField::Desired => {
                            if desired__.is_some() {
                                return Err(serde::de::Error::duplicate_field("desired"));
                            }
                            desired__ = map_.next_value()?;
                        }
                        GeneratedField::Results => {
                            if results__.is_some() {
                                return Err(serde::de::Error::duplicate_field("results"));
                            }
                            results__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = map_.next_value()?;
                        }
                        GeneratedField::Requirements => {
                            if requirements__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requirements"));
                            }
                            requirements__ = map_.next_value()?;
                        }
                        GeneratedField::Conditions => {
                            if conditions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("conditions"));
                            }
                            conditions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map_.next_value()?;
                        }
                    }
                }
                Ok(RunFunctionResponse {
                    meta: meta__,
                    desired: desired__,
                    results: results__.unwrap_or_default(),
                    context: context__,
                    requirements: requirements__,
                    conditions: conditions__.unwrap_or_default(),
                    output: output__,
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.RunFunctionResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Schema {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.openapi_v3.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.Schema", len)?;
        if let Some(v) = self.openapi_v3.as_ref() {
            struct_ser.serialize_field("openapiV3", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Schema {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "openapi_v3",
            "openapiV3",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OpenapiV3,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "openapiV3" | "openapi_v3" => Ok(GeneratedField::OpenapiV3),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Schema;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.Schema")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Schema, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut openapi_v3__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OpenapiV3 => {
                            if openapi_v3__.is_some() {
                                return Err(serde::de::Error::duplicate_field("openapiV3"));
                            }
                            openapi_v3__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Schema {
                    openapi_v3: openapi_v3__,
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.Schema", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SchemaSelector {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.api_version.is_empty() {
            len += 1;
        }
        if !self.kind.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.SchemaSelector", len)?;
        if !self.api_version.is_empty() {
            struct_ser.serialize_field("apiVersion", &self.api_version)?;
        }
        if !self.kind.is_empty() {
            struct_ser.serialize_field("kind", &self.kind)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SchemaSelector {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "api_version",
            "apiVersion",
            "kind",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ApiVersion,
            Kind,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "apiVersion" | "api_version" => Ok(GeneratedField::ApiVersion),
                            "kind" => Ok(GeneratedField::Kind),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SchemaSelector;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.SchemaSelector")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SchemaSelector, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut api_version__ = None;
                let mut kind__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ApiVersion => {
                            if api_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiVersion"));
                            }
                            api_version__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Kind => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("kind"));
                            }
                            kind__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SchemaSelector {
                    api_version: api_version__.unwrap_or_default(),
                    kind: kind__.unwrap_or_default(),
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.SchemaSelector", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Severity {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SEVERITY_UNSPECIFIED",
            Self::Fatal => "SEVERITY_FATAL",
            Self::Warning => "SEVERITY_WARNING",
            Self::Normal => "SEVERITY_NORMAL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Severity {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SEVERITY_UNSPECIFIED",
            "SEVERITY_FATAL",
            "SEVERITY_WARNING",
            "SEVERITY_NORMAL",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = Severity;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SEVERITY_UNSPECIFIED" => Ok(Severity::Unspecified),
                    "SEVERITY_FATAL" => Ok(Severity::Fatal),
                    "SEVERITY_WARNING" => Ok(Severity::Warning),
                    "SEVERITY_NORMAL" => Ok(Severity::Normal),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for State {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.composite.is_some() {
            len += 1;
        }
        if !self.resources.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("apiextensions.r#fn.proto.v1.State", len)?;
        if let Some(v) = self.composite.as_ref() {
            struct_ser.serialize_field("composite", v)?;
        }
        if !self.resources.is_empty() {
            struct_ser.serialize_field("resources", &self.resources)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for State {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "composite",
            "resources",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Composite,
            Resources,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl serde::de::Visitor<'_> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "composite" => Ok(GeneratedField::Composite),
                            "resources" => Ok(GeneratedField::Resources),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = State;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct apiextensions.r#fn.proto.v1.State")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<State, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut composite__ = None;
                let mut resources__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Composite => {
                            if composite__.is_some() {
                                return Err(serde::de::Error::duplicate_field("composite"));
                            }
                            composite__ = map_.next_value()?;
                        }
                        GeneratedField::Resources => {
                            if resources__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resources"));
                            }
                            resources__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(State {
                    composite: composite__,
                    resources: resources__.unwrap_or_default(),
                    unknown_fields: Default::default(),
                })
            }
        }
        deserializer.deserialize_struct("apiextensions.r#fn.proto.v1.State", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Status {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::ConditionUnspecified => "STATUS_CONDITION_UNSPECIFIED",
            Self::ConditionUnknown => "STATUS_CONDITION_UNKNOWN",
            Self::ConditionTrue => "STATUS_CONDITION_TRUE",
            Self::ConditionFalse => "STATUS_CONDITION_FALSE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Status {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "STATUS_CONDITION_UNSPECIFIED",
            "STATUS_CONDITION_UNKNOWN",
            "STATUS_CONDITION_TRUE",
            "STATUS_CONDITION_FALSE",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = Status;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "STATUS_CONDITION_UNSPECIFIED" => Ok(Status::ConditionUnspecified),
                    "STATUS_CONDITION_UNKNOWN" => Ok(Status::ConditionUnknown),
                    "STATUS_CONDITION_TRUE" => Ok(Status::ConditionTrue),
                    "STATUS_CONDITION_FALSE" => Ok(Status::ConditionFalse),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Target {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "TARGET_UNSPECIFIED",
            Self::Composite => "TARGET_COMPOSITE",
            Self::CompositeAndClaim => "TARGET_COMPOSITE_AND_CLAIM",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Target {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TARGET_UNSPECIFIED",
            "TARGET_COMPOSITE",
            "TARGET_COMPOSITE_AND_CLAIM",
        ];

        struct GeneratedVisitor;

        impl serde::de::Visitor<'_> for GeneratedVisitor {
            type Value = Target;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "TARGET_UNSPECIFIED" => Ok(Target::Unspecified),
                    "TARGET_COMPOSITE" => Ok(Target::Composite),
                    "TARGET_COMPOSITE_AND_CLAIM" => Ok(Target::CompositeAndClaim),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
