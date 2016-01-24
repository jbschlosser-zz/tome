use resin::{Datum, Interpreter, RuntimeError};
use tome::formatted_string::{self, Color, FormattedString};

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum ScriptAction {
    ReloadConfig,
    WriteScrollback(FormattedString),
    SendInput(String)
}

pub fn init_interpreter() -> Interpreter {
    let mut interp = Interpreter::new();
    interp.with_root(|root| {
        root.define_fn("tome:reload-config", |args: &[Datum]| {
            expect_args!(args == 0);
            Ok(Datum::ext(ScriptAction::ReloadConfig, "action:reload-config"))
        });
        root.define_fn("tome:write-scrollback", |args: &[Datum]| {
            expect_args!(args == 1);
            let s = try_unwrap_arg!(args[0] => String);
            let fs = formatted_string::with_color(&s, Color::Default);
            Ok(Datum::ext(ScriptAction::WriteScrollback(fs),
                          "action:write-scrollback"))
        });
        root.define_fn("tome:send", |args: &[Datum]| {
            expect_args!(args == 1);
            let s = try_unwrap_arg!(args[0] => String).clone();
            Ok(Datum::ext(ScriptAction::SendInput(s), "action:send"))
        });
    });
    interp
}
