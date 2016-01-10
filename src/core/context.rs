use session::Session;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Context<'a> {
    pub sessions: Vec<Session>,
    pub session_index: usize,
    pub bindings: HashMap<Vec<u8>, Rc<Box<Fn(&mut Context) -> bool + 'a>>>,
    pub key_codes_to_names: HashMap<Vec<u8>, String>,
    pub key_names_to_codes: HashMap<String, Vec<u8>>
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context {sessions: Vec::new(), session_index: 0,
            bindings: HashMap::new(), key_codes_to_names: HashMap::new(),
            key_names_to_codes: HashMap::new()}
    }
    pub fn current_session(&mut self) -> &mut Session {
        &mut self.sessions[self.session_index]
    }
    pub fn do_binding(&mut self, key: &Vec<u8>) -> Option<bool> {
        let binding = match self.bindings.get(key) {
            Some(b) => b.clone(),
            None => return None
        };
        Some(binding(self))
    }
    pub fn bind_key<F: Fn(&mut Context) -> bool + 'a>(&mut self,
        key_name: &str, func: F)
    {
        let code = match self.key_names_to_codes.get(key_name) {
            Some(c) => c.clone(),
            None => return
        };
        self.bind_keycode(code, func)
    }
    pub fn bind_keycode<F: Fn(&mut Context) -> bool + 'a>(&mut self,
        keycode: Vec<u8>, func: F)
    {
        self.bindings.insert(keycode, Rc::new(Box::new(func)));
    }
}
