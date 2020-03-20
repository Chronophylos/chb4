use humantime::format_duration;
use std::time::Instant;

pub struct Stopwatch<F>
where
    F: Fn(String),
{
    instant: Instant,
    callback: F,
}

impl<F> Stopwatch<F>
where
    F: Fn(String),
{
    pub fn new(callback: F) -> Self {
        Self {
            instant: Instant::now(),
            callback,
        }
    }
}

impl<F> Drop for Stopwatch<F>
where
    F: Fn(String),
{
    fn drop(&mut self) {
        (self.callback)(format_duration(self.instant.elapsed()).to_string())
    }
}
