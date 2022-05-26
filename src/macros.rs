macro_rules! ignore_results {
    ( $e:expr ) => {
        match $e {
            Ok(_) => (),
            Err(_) => (),
        }
    }
}
pub(crate) use ignore_results;
