use std::{ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::fuel_core::FuelCoreBlockHeight;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash, Copy)]
pub struct BlockHeight(u64);

impl Default for BlockHeight {
    fn default() -> Self {
        0.into()
    }
}

impl From<&FuelCoreBlockHeight> for BlockHeight {
    fn from(value: &FuelCoreBlockHeight) -> Self {
        value.to_owned().into()
    }
}

impl From<FuelCoreBlockHeight> for BlockHeight {
    fn from(value: FuelCoreBlockHeight) -> Self {
        let height = *value;
        BlockHeight(height as u64)
    }
}

impl std::fmt::Display for BlockHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for BlockHeight {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let height = s.parse::<u64>().map_err(|_| "Invalid block height")?;
        Ok(BlockHeight(height))
    }
}

impl Deref for BlockHeight {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<u64> for BlockHeight {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

macro_rules! impl_block_height_conversion {
    ($($t:ty),*) => {
        $(
            impl From<$t> for BlockHeight {
                fn from(value: $t) -> Self {
                    BlockHeight(value as u64)
                }
            }

            impl From<BlockHeight> for $t {
                fn from(value: BlockHeight) -> Self {
                    value.0 as $t
                }
            }

        )*
    };
}

impl BlockHeight {
    // Helper method to parse the internal string to u64
    fn as_number(&self) -> u64 {
        self.0
    }
}

impl PartialOrd for BlockHeight {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockHeight {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_number().cmp(&other.as_number())
    }
}

impl_block_height_conversion!(u32, i32, u64, i64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_height_conversions() {
        // Test u32 conversions
        let height_u32 = BlockHeight::from(100u32);
        assert_eq!(height_u32.0, 100);
        assert_eq!(u32::from(height_u32), 100u32);

        // Test i32 conversions
        let height_i32 = BlockHeight::from(200i32);
        assert_eq!(height_i32.0, 200);
        assert_eq!(i32::from(height_i32), 200i32);

        // Test u64 conversions
        let height_u64 = BlockHeight::from(300u64);
        assert_eq!(height_u64.0, 300);
        assert_eq!(u64::from(height_u64), 300u64);

        // Test i64 conversions
        let height_i64 = BlockHeight::from(400i64);
        assert_eq!(height_i64.0, 400);
        assert_eq!(i64::from(height_i64), 400i64);
    }

    #[test]
    fn test_block_height_comparisons() {
        let height1 = BlockHeight::from(100u32);
        let height2 = BlockHeight::from(200u32);
        let height3 = BlockHeight::from(200u32);

        // Test ordering
        assert!(height1 < height2);
        assert!(height2 >= height1);
        assert!(height2 > height1);
        assert!(height1 <= height2);

        // Test equality
        assert!(height2 == height3);
        assert!(height1 != height2);

        // Test combined comparisons
        assert!(height2 >= height3);
        assert!(height2 > height1);
        assert!(height1 < height2);
    }

    #[test]
    fn test_block_height_from_str() {
        // Test valid conversion
        let height = BlockHeight::from_str("100").unwrap();
        assert_eq!(height.0, 100);

        // Test invalid conversion
        assert!(BlockHeight::from_str("invalid").is_err());
    }

    #[test]
    fn test_block_height_display() {
        let height = BlockHeight::from(100u32);
        assert_eq!(format!("{}", height), "100");
    }
}
