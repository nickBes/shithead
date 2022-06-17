#[macro_export]
macro_rules! some_or_return {
    ($e:expr) => {
        match $e{
            Some(v)=>v,
            None=>return
        }
    };
}

