mod test;
use super::action::Action;

pub fn test<'a>() -> Action<'a> {
    test::action()
}
