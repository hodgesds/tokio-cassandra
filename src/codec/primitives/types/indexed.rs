#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlString {
    pub at: usize,
    pub len: u16,
}
