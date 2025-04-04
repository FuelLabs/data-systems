use fuel_streams_types::{
    declare_integer_wrapper,
    declare_string_wrapper,
    impl_utoipa_for_integer_wrapper,
};

declare_integer_wrapper!(ApiKeyId, u32);
declare_integer_wrapper!(ApiKeyRoleId, u32);
declare_integer_wrapper!(SubscriptionCount, u32);
declare_integer_wrapper!(RateLimitPerMinute, u32);
declare_integer_wrapper!(HistoricalLimit, u32);
declare_string_wrapper!(ApiKeyUserName);
declare_string_wrapper!(ApiKeyValue);

impl_utoipa_for_integer_wrapper!(ApiKeyId, "ApiKeyId", 0, u32::MAX as usize);

impl_utoipa_for_integer_wrapper!(
    ApiKeyRoleId,
    "ApiKeyRoleId",
    0,
    u32::MAX as usize
);

impl_utoipa_for_integer_wrapper!(
    SubscriptionCount,
    "SubscriptionCount",
    0,
    u32::MAX as usize
);

impl_utoipa_for_integer_wrapper!(
    RateLimitPerMinute,
    "RateLimitPerMinute",
    0,
    u32::MAX as usize
);

impl_utoipa_for_integer_wrapper!(
    HistoricalLimit,
    "HistoricalLimit",
    0,
    u32::MAX as usize
);
