#[macro_export]
macro_rules! cql_bytes {
    ($($b : expr), *) => {
        CqlBytes::try_from(vec![$($b), *]).unwrap()
    };
}

#[macro_export]
macro_rules! cql_string {
    ($s:expr) => {
        CqlString::try_from($s).unwrap()
    };
}
