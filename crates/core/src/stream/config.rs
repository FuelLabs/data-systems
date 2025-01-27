use std::sync::LazyLock;

pub static STREAM_THROTTLE_HISTORICAL: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("STREAM_THROTTLE_HISTORICAL")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(150)
});

pub static STREAM_THROTTLE_LIVE: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("STREAM_THROTTLE_LIVE")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(0)
});
