mod action_event_handler;
pub mod builder;
pub mod events;

use std::sync::Arc;

use self::builder::Builder;
use self::events::EventHandler;

pub fn listen(url: impl Into<String>) -> Builder {
    Builder::new(url)
}
