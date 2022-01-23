#[macro_export]
macro_rules! command_map {
    ( $( ($x:expr, $y:expr) ),* ) => {
        {

            let mut keys = std::collections::HashMap::<$crate::key::KeyPair, $crate::window_manager::Command>::new();

            $(
                keys.insert($x, $y);
            )*

            keys
        }
    };
}
