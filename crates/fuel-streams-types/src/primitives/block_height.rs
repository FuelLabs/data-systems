use crate::{declare_integer_wrapper, fuel_core::FuelCoreBlockHeight};

#[derive(thiserror::Error, Debug)]
pub enum BlockHeightError {
    #[error("Failed to parse to block_height: {0}")]
    InvalidFormat(String),
}

declare_integer_wrapper!(BlockHeight, u64, BlockHeightError);

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
        let height = BlockHeight::try_from("100").unwrap();
        assert_eq!(height.0, 100);

        // Test invalid conversion
        assert!(BlockHeight::try_from("invalid").is_err());
    }

    #[test]
    fn test_block_height_display() {
        let height = BlockHeight::from(100u32);
        assert_eq!(format!("{}", height), "100");
    }
}
