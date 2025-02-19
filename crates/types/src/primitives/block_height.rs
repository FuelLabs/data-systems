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
    use pretty_assertions::assert_eq;
    use serde_json;

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

    #[test]
    fn test_block_height_serialization() {
        let height = BlockHeight(7938487056892322000);
        let serialized = serde_json::to_string(&height).unwrap();
        let deserialized: BlockHeight =
            serde_json::from_str(&serialized).unwrap();
        assert_eq!(height, deserialized);

        // Test backwards compatibility with number format
        let json_number = "7938487056892322000";
        let deserialized: BlockHeight =
            serde_json::from_str(json_number).unwrap();
        assert_eq!(deserialized.0, 7938487056892322000);
    }

    #[test]
    fn test_block_height_option_conversions() {
        // From Option to BlockHeight
        let some_value = Some(42u64);
        let height: BlockHeight = some_value.into();
        assert_eq!(height.0, 42);

        let none_value: Option<u64> = None;
        let height: BlockHeight = none_value.into();
        assert_eq!(height.0, 0);

        // From BlockHeight to Option
        let height = BlockHeight(42);
        let option: Option<u64> = height.into();
        assert_eq!(option, Some(42));
    }

    #[test]
    fn test_block_height_null_deserialization() {
        // Test null
        let json_null = "null";
        let deserialized: BlockHeight =
            serde_json::from_str(json_null).unwrap();
        assert_eq!(deserialized, BlockHeight::default());

        // Test JSON with null field
        let json_obj = r#"{"height": null}"#;
        #[derive(serde::Deserialize)]
        struct Test {
            height: BlockHeight,
        }
        let deserialized: Test = serde_json::from_str(json_obj).unwrap();
        assert_eq!(deserialized.height, BlockHeight::default());
    }
}
