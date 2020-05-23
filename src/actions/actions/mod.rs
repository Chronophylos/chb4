use crate::actions::action::Action;
use std::sync::Arc;

mod prelude;

mod flamongo;
mod test;
mod voicemail;

pub fn all() -> Vec<Arc<Action>> {
    vec![test::action(), voicemail::action(), flamongo::action()]
}
