#[macro_export]
macro_rules! impl_conversions {
    ($name:ident, $inner_type:ty, $($t:ty),*) => {
        $(
            impl From<$t> for $name {
                fn from(value: $t) -> Self {
                    $name(value as $inner_type)
                }
            }

            impl From<$name> for $t {
                fn from(value: $name) -> Self {
                    value.0 as $t
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! declare_integer_wrapper {
    ($name:ident, $inner_type:ty, $error:ty) => {
        #[derive(
            Debug,
            Clone,
            Eq,
            PartialEq,
            serde::Serialize,
            serde::Deserialize,
            Hash,
            Copy,
        )]
        pub struct $name($inner_type);

        impl Default for $name {
            fn default() -> Self {
                0.into()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::ops::Deref for $name {
            type Target = $inner_type;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl AsRef<$inner_type> for $name {
            fn as_ref(&self) -> &$inner_type {
                &self.0
            }
        }

        impl $name {
            fn as_number(&self) -> $inner_type {
                self.0
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.as_number().cmp(&other.as_number())
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $error;
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                let value = s
                    .parse::<$inner_type>()
                    .map_err(|_| <$error>::InvalidFormat(s.to_string()))?;
                Ok($name(value))
            }
        }

        impl TryFrom<String> for $name {
            type Error = $error;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                s.as_str().try_into()
            }
        }

        // TODO: remove this and let just TryFrom<&str>
        impl std::str::FromStr for $name {
            type Err = $error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.try_into()
            }
        }

        $crate::impl_conversions!($name, $inner_type, u32, i32, u64, i64);
    };
}
