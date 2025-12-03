#[macro_export]
macro_rules! impl_enum_string_serialization {
    ($name:ident, $db_name:expr) => {
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                $name::try_from(s.as_str()).map_err(serde::de::Error::custom)
            }
        }

        impl FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $name::try_from(s)
            }
        }
    };
}
