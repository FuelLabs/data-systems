#[macro_export]
macro_rules! declare_string_wrapper {
    ($name:ident) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash, utoipa::ToSchema)]
        pub struct $name(std::borrow::Cow<'static, str>);

        impl $name {
            pub fn new<T: Into<std::borrow::Cow<'static, str>>>(value: T) -> Self {
                Self(value.into())
            }

            pub fn into_inner(self) -> std::borrow::Cow<'static, str> {
                self.0
            }

            pub fn as_str(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(std::borrow::Cow::Borrowed(""))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                self.0.as_ref()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.as_ref().cmp(other.0.as_ref())
            }
        }

        impl From<&'static str> for $name {
            fn from(s: &'static str) -> Self {
                Self(std::borrow::Cow::Borrowed(s))
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(std::borrow::Cow::Owned(s))
            }
        }

        impl From<std::borrow::Cow<'static, str>> for $name {
            fn from(s: std::borrow::Cow<'static, str>) -> Self {
                Self(s)
            }
        }

        impl From<$name> for String {
            fn from(wrapper: $name) -> Self {
                wrapper.0.into_owned()
            }
        }

        impl From<$name> for std::borrow::Cow<'static, str> {
            fn from(wrapper: $name) -> Self {
                wrapper.0
            }
        }

        impl From<Option<String>> for $name {
            fn from(value: Option<String>) -> Self {
                match value {
                    Some(v) => Self(std::borrow::Cow::Owned(v)),
                    None => Self::default(),
                }
            }
        }

        impl From<Option<&'static str>> for $name {
            fn from(value: Option<&'static str>) -> Self {
                match value {
                    Some(v) => Self(std::borrow::Cow::Borrowed(v)),
                    None => Self::default(),
                }
            }
        }

        impl From<$name> for Option<String> {
            fn from(wrapper: $name) -> Self {
                if wrapper.0.is_empty() {
                    None
                } else {
                    Some(wrapper.0.into_owned())
                }
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.0.as_ref())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct StringVisitor;

                impl<'de> serde::de::Visitor<'de> for StringVisitor {
                    type Value = $name;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter,
                    ) -> std::fmt::Result {
                        formatter.write_str("a string or null")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name(std::borrow::Cow::Owned(value.to_owned())))
                    }

                    fn visit_string<E>(self, value: String) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name(std::borrow::Cow::Owned(value)))
                    }

                    fn visit_none<E>(self) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name::default())
                    }

                    fn visit_unit<E>(self) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name::default())
                    }
                }

                deserializer.deserialize_any(StringVisitor)
            }
        }

        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                <String as sqlx::Type<sqlx::Postgres>>::type_info()
            }

            fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
                *ty == sqlx::postgres::PgTypeInfo::with_name("TEXT")
                    || *ty == sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
                Ok($name::new(value))
            }
        }

        impl sqlx::Encode<'_, sqlx::Postgres> for $name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
            }
        }
    };
}

#[derive(thiserror::Error, Debug)]
pub enum WrappedStrError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

// Example usage of the macro
declare_string_wrapper!(WrappedStr);

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use pretty_assertions::assert_eq;
    use serde_json::{
        from_str,
        to_string,
    };

    use super::*;

    #[test]
    fn test_default() {
        let default_str = WrappedStr::default();
        assert_eq!(default_str.as_str(), "");
        assert!(default_str.is_empty());
    }

    #[test]
    fn test_from_static_str() {
        let static_str = WrappedStr::from("hello");
        assert_eq!(static_str.as_str(), "hello");
        assert!(matches!(static_str.0, Cow::Borrowed(_)));
    }

    #[test]
    fn test_from_string() {
        let string_str = WrappedStr::from(String::from("hello"));
        assert_eq!(string_str.as_str(), "hello");
        assert!(matches!(string_str.0, Cow::Owned(_)));
    }

    #[test]
    fn test_from_cow() {
        let borrowed_cow = Cow::Borrowed("hello");
        let owned_cow = Cow::Owned(String::from("world"));

        let borrowed_str = WrappedStr::from(borrowed_cow);
        let owned_str = WrappedStr::from(owned_cow);

        assert_eq!(borrowed_str.as_str(), "hello");
        assert_eq!(owned_str.as_str(), "world");
    }

    #[test]
    fn test_into_string() {
        let wrapped = WrappedStr::from("hello");
        let string: String = wrapped.into();
        assert_eq!(string, "hello");
    }

    #[test]
    fn test_into_cow() {
        let wrapped = WrappedStr::from("hello");
        let cow: Cow<'static, str> = wrapped.into();
        assert_eq!(cow, "hello");
    }

    #[test]
    fn test_from_option_string() {
        let some_string = WrappedStr::from(Some(String::from("hello")));
        let none_string: WrappedStr = WrappedStr::from(None::<String>);

        assert_eq!(some_string.as_str(), "hello");
        assert_eq!(none_string.as_str(), "");
    }

    #[test]
    fn test_from_option_static_str() {
        let some_str = WrappedStr::from(Some("hello"));
        let none_str: WrappedStr = WrappedStr::from(None::<&'static str>);

        assert_eq!(some_str.as_str(), "hello");
        assert_eq!(none_str.as_str(), "");
    }

    #[test]
    fn test_into_option_string() {
        let wrapped = WrappedStr::from("hello");
        let empty = WrappedStr::default();

        let option_some: Option<String> = wrapped.into();
        let option_none: Option<String> = empty.into();

        assert_eq!(option_some, Some(String::from("hello")));
        assert_eq!(option_none, None);
    }

    #[test]
    fn test_from_str() {
        let value: WrappedStr = "hello".into();
        assert_eq!(value.as_str(), "hello");
    }

    #[test]
    fn test_display() {
        let wrapped = WrappedStr::from("hello");
        assert_eq!(format!("{}", wrapped), "hello");
    }

    #[test]
    fn test_deref() {
        let wrapped = WrappedStr::from("hello world");
        assert_eq!(wrapped.len(), 11);
        assert!(wrapped.contains("world"));
    }

    #[test]
    fn test_as_ref() {
        let wrapped = WrappedStr::from("hello");
        let s: &str = wrapped.as_ref();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_ordering() {
        let a = WrappedStr::from("a");
        let b = WrappedStr::from("b");
        let a2 = WrappedStr::from("a");

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, a2);
        assert!(a <= a2);
        assert!(a >= a2);
    }

    #[test]
    fn test_serde() {
        let wrapped = WrappedStr::from("hello");

        // Test serialization
        let serialized = to_string(&wrapped).unwrap();
        assert_eq!(serialized, "\"hello\"");

        // Test deserialization
        let deserialized: WrappedStr = from_str("\"hello\"").unwrap();
        assert_eq!(deserialized, wrapped);

        // Test null deserialization
        let null_deserialized: WrappedStr = from_str("null").unwrap();
        assert_eq!(null_deserialized, WrappedStr::default());
    }

    #[test]
    fn test_new_method() {
        let static_str = WrappedStr::new("hello");
        let string = WrappedStr::new(String::from("world"));
        let cow_borrowed = WrappedStr::new(Cow::Borrowed("borrowed"));
        let cow_owned = WrappedStr::new(Cow::Owned(String::from("owned")));

        assert_eq!(static_str.as_str(), "hello");
        assert_eq!(string.as_str(), "world");
        assert_eq!(cow_borrowed.as_str(), "borrowed");
        assert_eq!(cow_owned.as_str(), "owned");
    }

    #[test]
    fn test_into_inner() {
        let wrapped = WrappedStr::from("hello");
        let cow = wrapped.into_inner();
        assert_eq!(cow, "hello");
    }
}
