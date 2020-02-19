use super::action::Action;
use crate::context::Context;
use std::sync::Arc;

mod test;

pub fn all(_context: Arc<Context>) -> Vec<Action> {
    vec![test::action()]
}
