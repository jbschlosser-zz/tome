use session::Session;
use std::collections::HashMap;

pub struct Context {
    pub sessions: Vec<Session>,
    pub session_index: usize,
    pub bindings: HashMap<i32, Box<FnMut(&mut Session) -> bool>>,
}

impl Context {
    pub fn new() -> Context {
        Context {sessions: Vec::new(), session_index: 0, bindings: HashMap::new()}
    }
    pub fn get_current_session(&mut self) -> &mut Session {
        &mut self.sessions[self.session_index]
    }
    pub fn do_binding(&mut self, key: i32) -> Option<bool> {
        match self.bindings.get_mut(&key) {
            Some(binding) => {
                Some(binding(&mut self.sessions[self.session_index]))
            },
            None => None
        }
    }
}

