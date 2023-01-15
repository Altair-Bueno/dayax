mod dayax;
mod handler;
mod request;
mod response;

use std::sync::Arc;
use tokio::sync::Mutex;

pub use crate::dayax::Dayax;
use axum::Router;
use mlua::Lua;

/// Type alias for [`axum::Router`] that has `DayaxState` as state
pub type DayaxRouter = Router<DayaxState>;
/// Type alias for `Arc<Mutex<Lua>>`
pub type DayaxState = Arc<Mutex<Lua>>;
