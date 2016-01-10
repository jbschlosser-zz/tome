use super::actions;
use resin::{Datum, Environment, Interpreter, RuntimeError};

pub fn get_interpreter() -> Interpreter {
    let mut interp = Interpreter::new();
    interp.with_root(|root: &mut Environment| {
        root.define("x", Datum::Number(2));
    });
    interp
}
