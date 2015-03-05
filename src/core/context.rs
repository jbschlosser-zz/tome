use session::Session;
use std::collections::HashMap;

pub struct Context {
    pub sessions: Vec<Session>,
    pub session_index: Option<usize>,
    pub bindings: HashMap<i64, Box<FnMut(&mut Session)>>,
}

impl Context {
    pub fn new() -> Context {
        Context {sessions: Vec::new(), session_index: None, bindings: HashMap::new()}
    }
}

