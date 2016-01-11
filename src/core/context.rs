use actions;
use formatted_string::FormattedString;
use indexed::Indexed;
use keys::get_key_codes_to_names;
use super::resin;
use ring_buffer::RingBuffer;
use session::Session;
use scripting;
use std::char;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Context<'a> {
    pub sessions: Vec<Session>,
    pub session_index: usize,
    pub bindings: HashMap<Vec<u8>, Rc<Box<Fn(&mut Context) -> bool + 'a>>>,
    pub key_codes_to_names: HashMap<Vec<u8>, String>,
    pub key_names_to_codes: HashMap<String, Vec<u8>>,
    pub history: Indexed<RingBuffer<FormattedString>>,
    pub cursor_index: usize,
    pub interpreter: resin::Interpreter
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        let key_codes_to_names = get_key_codes_to_names();
        let mut key_names_to_codes = HashMap::new();
        for (code, name) in key_codes_to_names.iter() {
            key_names_to_codes.insert(name.clone(), code.clone());
        }
        let mut history = Indexed::<_>::new(RingBuffer::new(None));
        history.data.push(FormattedString::new());
        let mut context = Context {
            sessions: Vec::new(),
            session_index: 0,
            bindings: HashMap::new(),
            key_codes_to_names: key_codes_to_names,
            key_names_to_codes: key_names_to_codes,
            history: history,
            cursor_index: 0,
            interpreter: scripting::get_interpreter()
        };
        context.set_default_bindings();
        context
    }
    pub fn current_session(&self) -> &Session {
        &self.sessions[self.session_index]
    }
    pub fn current_session_mut(&mut self) -> &mut Session {
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
    fn set_default_bindings(&mut self) {
        self.bind_key("F12", actions::quit);
        self.bind_key("PAGEUP", actions::prev_page);
        self.bind_key("PAGEDOWN", actions::next_page);
        self.bind_key("BACKSPACE", actions::backspace_input);
        self.bind_key("DELETE", actions::delete_input_char);
        self.bind_key("ENTER", actions::send_input);
        self.bind_keycode(vec![13], actions::send_input);
        self.bind_key("LEFT", actions::cursor_left);
        self.bind_key("RIGHT", actions::cursor_right);
        self.bind_key("UP", actions::history_prev);
        self.bind_key("DOWN", actions::history_next);
        // Ctrl-U.
        self.bind_keycode(vec![21], actions::delete_to_cursor);

        // Keys that should be displayed directly.
        for i in 0x20u8..0x7Fu8 {
            let name = (i as char).to_string();
            self.bind_key(&name, move |context: &mut Context| {
                let ch = char::from_u32(i as u32).unwrap();
                actions::insert_input_char(context, ch);
                true
            });
        }
    }
}
