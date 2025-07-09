use std::{
    borrow::Cow,
    fmt::{self, Display},
    ops::Deref,
};

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, utoipa::ToSchema,
)]
pub struct Cursor(Cow<'static, str>);

impl Cursor {
    // We don't pad the first element (`block_height`)
    const CURSOR_PAD_FROM: usize = 1;
    const CURSOR_PAD_LENGTH: usize = 6;

    pub fn new(fields: &[&dyn ToString]) -> Self {
        Self(Cow::Owned(
            fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    if i < Self::CURSOR_PAD_FROM {
                        f.to_string()
                    } else {
                        format!(
                            "{:0>width$}",
                            f.to_string(),
                            width = Self::CURSOR_PAD_LENGTH
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join("-"),
        ))
    }

    pub fn from_static(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl Display for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Cursor {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&'static str> for Cursor {
    fn from(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl From<String> for Cursor {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl From<Cursor> for String {
    fn from(cursor: Cursor) -> Self {
        cursor.0.into_owned()
    }
}

impl AsRef<str> for Cursor {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for Cursor {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cursor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
