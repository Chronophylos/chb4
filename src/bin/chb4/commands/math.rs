use super::prelude::*;
use evalexpr::*;
use std::f64::consts;

pub fn command() -> Arc<Command> {
    Command::with_name("math")
        .alias("quickmafs")
        .command(move |_context, args, _msg, _user| {
            let context = context_map! {
                "e" => consts::E,
                "pi" => consts::PI,
                "π" => consts::PI,
                "sqrt" => Function::new(Box::new(|argument| {
                    if let Ok(int) = argument.as_int() {
                        Ok(Value::Float((int as f64).sqrt()))
                    } else if let Ok(float) = argument.as_float() {
                        Ok(Value::Float(float.sqrt()))
                    } else {
                        Err(EvalexprError::expected_number(argument.clone()))
                    }
                })),
                "abs" => Function::new(Box::new(|argument| {
                    if let Ok(int) = argument.as_int() {
                        Ok(Value::Int(int.abs()))
                    } else if let Ok(float) = argument.as_float() {
                        Ok(Value::Float(float.abs()))
                    } else {
                        Err(EvalexprError::expected_number(argument.clone()))
                    }
                }))
            }
            .unwrap();

            let expr = args.join(" ");
            Ok(match eval_with_context(&expr, &context) {
                Ok(s) => MessageResult::Message(format!("{}", s)),
                Err(e) => MessageResult::Message(format!("Error: {}", e)),
            })
        })
        .about("Do some math")
        .description(
            "do math.

USAGE: math <expr>
       quickmafs <expr>

        where <expr> is a valid mathematical expression.
",
        )
        .done()
}
