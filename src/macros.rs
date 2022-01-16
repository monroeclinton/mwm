#[macro_export]
macro_rules! key_map {
    ( $( ($x:expr, $y:expr) ),* ) => {
        {
            use std::collections::HashMap;
            let mut keys = HashMap::<$crate::key::KeyPair, $crate::config::Command>::new();

            $(
                keys.insert($x, $y);
            )*

            keys
        }
    };
}
