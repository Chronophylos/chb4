use super::prelude::*;
use evalexpr::*;
use std::f64::consts;

static PHI: f64 = 1.61803398874989484820;

pub fn command() -> Arc<Command> {
    Command::with_name("math")
        .alias("quickmafs")
        .command(move |_context, args, _msg, _user| {
            // TODO: cache context
            let context = context_map! {
                "e" => consts::E,
                "π" => consts::PI,
                "pi" => consts::PI,
                "phi" => PHI,
                "φ" => PHI,
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
                })),
                "floor"=> Function::new(Box::new(|argument| {
                    if let Ok(int) = argument.as_int() {
                        Ok(Value::Int(int))
                    } else if let Ok(float) = argument.as_float() {
                        Ok(Value::Float(float.floor()))
                    } else {
                        Err(EvalexprError::expected_number(argument.clone()))
                    }
                })),
                "ceil"=> Function::new(Box::new(|argument| {
                    if let Ok(int) = argument.as_int() {
                        Ok(Value::Int(int))
                    } else if let Ok(float) = argument.as_float() {
                        Ok(Value::Float(float.ceil()))
                    } else {
                        Err(EvalexprError::expected_number(argument.clone()))
                    }
                })),
                "round"=> Function::new(Box::new(|argument| {
                    if let Ok(int) = argument.as_int() {
                        Ok(Value::Int(int))
                    } else if let Ok(float) = argument.as_float() {
                        Ok(Value::Float(float.round()))
                    } else {
                        Err(EvalexprError::expected_number(argument.clone()))
                    }
                }))
            }
            .unwrap();

            let expr = args.join(" ");
            Ok(match eval_with_context(&expr, &context) {
                Ok(s) => MessageResult::Message(format!("{}", s)),
                Err(err) => MessageResult::Error(err.to_string()),
            })
        })
        .about("Do some math")
        .description(
            "
This command uses the `evalexpr` crate. This crate allows the definition of constants and functions.

.Constants
|===
| Name | Value | Description

| e
| 2.71828182845904523536028747135266250f64
| Euler's number (e)

| pi, π
| 3.14159265358979323846264338327950288f64
| Archimedes' constant (π)
|===

.Functions
|===
| Name | Description

| sqrt(x)
| Square root of x

| abs(x)
| Absolute value of x

| floor(x)
| Returns the largest integer less than or equal to a number.

| ceil(x)
| Returns the smallest integer greater than or equal to a number.

| round(x)
| Returns the nearest integer to a number. Round half-way cases away from `0.0`.
|===

==== USAGE

```
math <expr>
```
Where <expr> is a valid mathematical expression.
",
        )
        .done()
}
