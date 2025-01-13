pub(crate) mod close;
pub(crate) mod handler_impl;
pub(crate) mod message;
pub(crate) mod ping_pong;
pub(crate) mod subscription;
pub(crate) mod unknown;

pub use handler_impl::*;

#[macro_export]
macro_rules! handle_ws_error {
    ($result:expr, $ctx:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                $ctx.close_with_error(e.into()).await;
                return Ok(());
            }
        }
    };
}
