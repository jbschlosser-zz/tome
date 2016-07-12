mod resin_interface;

use tome::formatted_string::FormattedString;
use self::resin_interface::ResinScriptInterface;

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum ScriptAction {
    ReloadConfig,
    WriteScrollback(FormattedString),
    SendInput(String),
    Reconnect,
    SearchBackwards(String)
}

pub trait ScriptInterface {
    fn send_hook(&mut self, input: &str) ->
        Result<Vec<ScriptAction>, String>;
    fn recv_hook(&mut self, data: &FormattedString) ->
        Result<Vec<ScriptAction>, String>;
    fn evaluate(&mut self, s: &str) -> Result<(), String>;
}

pub fn init_interface() -> Box<ScriptInterface> {
    Box::new(ResinScriptInterface::new()) as Box<ScriptInterface>
}
