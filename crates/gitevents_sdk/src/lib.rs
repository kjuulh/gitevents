mod action_event_handler;
pub mod builder;
pub mod cron;
pub mod events;
pub mod git;

use self::builder::Builder;

pub fn listen(url: impl Into<String>) -> Builder {
    Builder::new().set_generic_git_url(url)
}
