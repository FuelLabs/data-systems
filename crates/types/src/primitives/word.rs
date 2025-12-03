use crate::{
    declare_integer_wrapper,
    impl_avro_schema_for_wrapped_int,
    impl_utoipa_for_integer_wrapper,
};

declare_integer_wrapper!(Word, u64);
impl_avro_schema_for_wrapped_int!(Word, u64);

impl_utoipa_for_integer_wrapper!(Word, "A word in the blockchain", 0, u64::MAX as usize);
