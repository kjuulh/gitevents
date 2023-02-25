mod action_event_handler;
pub mod builder;
pub mod events;



use self::builder::Builder;


pub fn listen(url: impl Into<String>) -> Builder {
    Builder::new(url)
}
