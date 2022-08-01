use set_timeout::TimeoutScheduler;

lazy_static::lazy_static! {
    pub static ref TIMEOUT_SCHEDULER: TimeoutScheduler = TimeoutScheduler::new(None);
}

#[macro_export]
macro_rules! some_or_return {
    ($e:expr) => {
        match $e{
            Some(v)=>v,
            None=>return
        }
    };
}

