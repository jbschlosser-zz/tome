use super::actions;
use resin::{Datum, Environment, Interpreter, RuntimeError};

pub fn get_interpreter() -> Interpreter {
    let mut interp = Interpreter::new();
    interp.with_root(|root: &mut Environment| {
        root.define_fn("tome:reload-config", |args: &[Datum]| {
            expect_args!(args == 0);
            // TODO: Get the Context into here somehow.
            //actions::reload_config();
            Ok(Datum::Boolean(true))
        });
    });
    interp
}
