#[derive(Debug, Eq, PartialEq)]
pub enum ParseState {
    NotInProgress,
    InProgress(Vec<u8>),
    Success(Vec<u8>),
    Error(Vec<u8>)
}
