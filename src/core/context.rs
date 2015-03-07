use session::Session;
use std::collections::HashMap;

pub struct Context {
    pub sessions: Vec<Session>,
    pub session_index: Option<usize>,
    pub bindings: HashMap<i32, Box<FnMut(&mut Session) -> bool>>,
}

impl Context {
    pub fn new() -> Context {
        Context {sessions: Vec::new(), session_index: None, bindings: HashMap::new()}
    }
    pub fn get_current_session(&mut self) -> Option<&mut Session> {
        match self.session_index {
            Some(s) => Some(&mut self.sessions[s]),
            None => None
        }
    }
    pub fn do_binding(&mut self, key: i32) -> Option<bool> {
        match self.bindings.get_mut(&key) {
            Some(binding) => {
                Some(binding(&mut self.sessions[self.session_index.unwrap()]))
            },
            None => None
        }
    }
}

