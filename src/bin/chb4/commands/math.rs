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
            "
This command uses the `evalexpr` crate. This crate allows the definition of constants and functions.

CONSTANTS:

|==
| Name | Value | Description

| e
| 2.71828182845904523536028747135266250f64
| Euler's number (e)

| pi, π
| 3.14159265358979323846264338327950288f64
| Archimedes' constant (π)
|==

FUNCTIONS:

|==
| Name | Description

| sqrt(x)
| Square root of x

| abs(x)
| Absolute value of x
|==
        
USAGE: math <expr>
        math <expr>

        where <expr> is a valid mathematical expression.
",
        )
        .done()
}
