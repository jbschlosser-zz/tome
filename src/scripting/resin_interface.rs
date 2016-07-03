use super::super::tome::formatted_string::{self, Color, FormattedString};
use resin::{Datum, Interpreter, RuntimeError};
use scripting::{ScriptAction, ScriptInterface};

pub struct ResinScriptInterface {
    interp: Interpreter
}

impl ResinScriptInterface {
    pub fn new() -> Self {
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

        ResinScriptInterface { interp: interp }
    }
}

impl ScriptInterface for ResinScriptInterface {
    fn send_input_hook(&mut self, input: &str) ->
        Result<Vec<ScriptAction>, String>
    {
        let hook = self.interp.root().get("send-input-hook");
        if let Some(h) = hook {
            // Evaluate the hook with the input.
            let expr = list!(h, Datum::String(String::from(input)));
            let eval_result = self.interp.evaluate_datum(&expr);
            match eval_result {
                Ok(d) => {
                    let mut actions = Vec::<ScriptAction>::new();
                    for da in d.as_vec().0.into_iter() {
                        match unwrap_arg!(da => ScriptAction) {
                            Ok(a) => actions.push(a),
                            Err(_) => return Err(String::from("Non-action returned"))
                        }
                    }
                    Ok(actions)
                },
                Err((e, trace)) => {
                    Err(format!("Script error: {}\n{}\n", &e.msg, &trace))
                }
            }
        } else {
            Ok(vec![ScriptAction::SendInput(String::from(input))])
        }
    }
    fn recv_data_hook(&mut self, _: &FormattedString) ->
        Result<Vec<ScriptAction>, String>
    {
        unimplemented!();
    }
    fn evaluate(&mut self, s: &str) -> Result<(), String>
    {
        match self.interp.evaluate(s) {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }
}
