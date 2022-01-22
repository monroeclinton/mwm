#[macro_export]
macro_rules! key_map {
    ( $( ($x:expr, $y:expr) ),* ) => {
        {
            use std::collections::HashMap;
            let mut keys = HashMap::<$crate::key::KeyPair, $crate::window_manager::Command>::new();

            $(
                keys.insert($x, $y);
            )*

            keys
        }
    };
}
