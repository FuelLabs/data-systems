/// Macro to implement `AvroSchemaComponent` for fixed-size byte array types
#[macro_export]
macro_rules! impl_avro_schema_for_fixed_bytes {
    ($type:ident, $size:expr) => {
        impl AvroSchemaComponent for $type {
            fn get_schema_in_ctxt(
                named_schemas: &mut HashMap<Name, Schema>,
                enclosing_namespace: &Namespace,
            ) -> Schema {
                // Create the fully qualified name for the type
                let name = Name::new(stringify!($type))
                    .unwrap()
                    .fully_qualified_name(enclosing_namespace);

                // If the schema is already defined, return a reference to it
                if named_schemas.contains_key(&name) {
                    return Schema::Ref { name: name.clone() };
                }

                // Insert a reference to prevent infinite loops
                named_schemas
                    .insert(name.clone(), Schema::Ref { name: name.clone() });

                // Create the actual fixed schema
                let schema = Schema::Fixed(FixedSchema {
                    name: name.clone(),
                    size: $size,
                    aliases: None,
                    attributes: BTreeMap::new(),
                    default: None,
                    doc: None,
                });

                // Update the entry with the actual schema
                named_schemas.insert(name, schema.clone());
                schema
            }
        }

        // Add custom AvroValue implementation
        impl TryFrom<$type> for apache_avro::types::Value {
            type Error = apache_avro::Error;

            fn try_from(value: $type) -> Result<Self, Self::Error> {
                let bytes: &[u8; $size] = &value.as_ref();
                Ok(apache_avro::types::Value::Fixed($size, bytes.to_vec()))
            }
        }

        impl TryFrom<apache_avro::types::Value> for $type {
            type Error = apache_avro::Error;

            fn try_from(
                value: apache_avro::types::Value,
            ) -> Result<Self, Self::Error> {
                let schema = Schema::Fixed(FixedSchema {
                    name: Name::new(stringify!($type)).unwrap(),
                    size: $size,
                    aliases: None,
                    attributes: BTreeMap::new(),
                    default: None,
                    doc: None,
                });

                match value {
                    apache_avro::types::Value::Fixed(size, bytes) => {
                        if size != $size {
                            return Err(
                                apache_avro::Error::ValidationWithReason {
                                    value: apache_avro::types::Value::Fixed(
                                        size, bytes,
                                    ),
                                    schema,
                                    reason: format!(
                                        "Expected fixed size {}, got {}",
                                        $size, size
                                    ),
                                },
                            );
                        }
                        let arr: [u8; $size] =
                            bytes.clone().try_into().map_err(|_| {
                                let bytes_len = bytes.len();
                                apache_avro::Error::ValidationWithReason {
                                    value: apache_avro::types::Value::Fixed(
                                        size, bytes,
                                    ),
                                    schema,
                                    reason: format!(
                                        "Expected {} bytes, got {}",
                                        $size, bytes_len,
                                    ),
                                }
                            })?;
                        Ok($type::from(arr))
                    }
                    _ => Err(apache_avro::Error::ValidationWithReason {
                        value,
                        schema,
                        reason: "Expected fixed bytes".to_string(),
                    }),
                }
            }
        }
    };
}

/// Macro to implement `AvroSchemaComponent` for variable-length byte array types
#[macro_export]
macro_rules! impl_avro_schema_for_variable_bytes {
    ($type:ident) => {
        impl AvroSchemaComponent for $type {
            fn get_schema_in_ctxt(
                _named_schemas: &mut HashMap<Name, Schema>,
                _enclosing_namespace: &Namespace,
            ) -> Schema {
                // For variable-length bytes, we can use Schema::Bytes directly
                Schema::Bytes
            }
        }
    };
}

#[macro_export]
macro_rules! generate_bool_type_wrapper {
    ($wrapper_type:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $wrapper_type(pub bool);

        impl serde::Serialize for $wrapper_type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_bool(self.0)
            }
        }

        impl<'de> serde::Deserialize<'de> for $wrapper_type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct BoolVisitor;

                impl<'de> serde::de::Visitor<'de> for BoolVisitor {
                    type Value = $wrapper_type;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter,
                    ) -> std::fmt::Result {
                        formatter.write_str(
                            "a boolean value or Bool(true)/Bool(false)",
                        )
                    }

                    // Handle plain boolean values
                    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($wrapper_type(v))
                    }

                    // Keep string handling for Bool(true)/Bool(false) format
                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        if v.starts_with("Bool(") && v.ends_with(")") {
                            let bool_str = &v[5..v.len() - 1];
                            match bool_str {
                                "true" => Ok($wrapper_type(true)),
                                "false" => Ok($wrapper_type(false)),
                                _ => Err(E::custom(format!(
                                    "Invalid boolean value: {}",
                                    v
                                ))),
                            }
                        } else {
                            Err(E::custom(format!(
                                "Expected Bool(true) or Bool(false), got {}",
                                v
                            )))
                        }
                    }
                }

                deserializer.deserialize_any(BoolVisitor)
            }
        }

        impl Default for $wrapper_type {
            fn default() -> Self {
                $wrapper_type(false)
            }
        }

        impl From<bool> for $wrapper_type {
            fn from(value: bool) -> Self {
                $wrapper_type(value)
            }
        }

        impl From<$wrapper_type> for bool {
            fn from(value: $wrapper_type) -> Self {
                value.0
            }
        }
    };
}

#[macro_export]
macro_rules! impl_avro_schema_for_bool {
    ($type:ident) => {
        impl AvroSchemaComponent for $type {
            fn get_schema_in_ctxt(
                _named_schemas: &mut HashMap<Name, Schema>,
                _enclosing_namespace: &Namespace,
            ) -> Schema {
                Schema::Boolean
            }
        }

        // Add custom AvroValue implementation
        impl TryFrom<$type> for apache_avro::types::Value {
            type Error = apache_avro::Error;

            fn try_from(value: $type) -> Result<Self, Self::Error> {
                Ok(apache_avro::types::Value::Boolean(value.0))
            }
        }

        impl TryFrom<apache_avro::types::Value> for $type {
            type Error = apache_avro::Error;

            fn try_from(
                value: apache_avro::types::Value,
            ) -> Result<Self, Self::Error> {
                match value {
                    apache_avro::types::Value::Boolean(b) => Ok($type(b)),
                    _ => Err(apache_avro::Error::ValidationWithReason {
                        value,
                        schema: apache_avro::Schema::Boolean,
                        reason: "Expected boolean value".to_string(),
                    }),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_avro_schema_for_wrapped_int {
    ($type:ident, $inner_type:ty) => {
        impl TryFrom<$type> for apache_avro::types::Value {
            type Error = apache_avro::Error;
            fn try_from(value: $type) -> Result<Self, Self::Error> {
                Ok(apache_avro::types::Value::Long(value.0 as i64))
            }
        }

        impl TryFrom<apache_avro::types::Value> for $type {
            type Error = apache_avro::Error;
            fn try_from(
                value: apache_avro::types::Value,
            ) -> Result<Self, Self::Error> {
                match value {
                    apache_avro::types::Value::Long(n) => {
                        Ok($type(n as $inner_type))
                    }
                    _ => Err(apache_avro::Error::Validation),
                }
            }
        }

        impl apache_avro::schema::derive::AvroSchemaComponent for $type {
            fn get_schema_in_ctxt(
                _ctxt: &mut std::collections::HashMap<
                    apache_avro::schema::Name,
                    apache_avro::schema::Schema,
                >,
                _namespace: &Option<String>,
            ) -> apache_avro::schema::Schema {
                // Use Avro's `long` type (i64) for serialization
                apache_avro::schema::Schema::Long
            }
        }
    };
}
