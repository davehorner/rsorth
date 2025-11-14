use std::rc::Rc;

use crate::runtime::data_structures::dictionary::{WordRuntime, WordType, WordVisibility};
use crate::runtime::data_structures::value::Value;
use crate::runtime::error::ScriptError;
use crate::runtime::interpreter::Interpreter;
use crate::add_native_word;

pub fn register_simple_arithmetic_words(interpreter: &mut dyn Interpreter) {
        // Forth-compatible multiply-divide: ( n1 n2 n3 -- n4 ) n1 n2 * n3 / (truncate)
        add_native_word!(
            interpreter,
            "*/",
            |interp: &mut dyn Interpreter| {
                let divisor = interp.pop_as_int()?;
                let b = interp.pop_as_int()?;
                let a = interp.pop_as_int()?;
                interp.push(Value::Int((a * b) / divisor));
                Ok(())
            },
            "( n1 n2 n3 -- n4 ) Multiply n1 and n2, then divide by n3 (truncate).",
            "n1 n2 n3 -- n4"
        );

        // Forth-compatible multiply-divide-mod: ( n1 n2 n3 -- n4 n5 ) n1 n2 * n3 /mod
        add_native_word!(
            interpreter,
            "*/mod",
            |interp: &mut dyn Interpreter| {
                let divisor = interp.pop_as_int()?;
                let b = interp.pop_as_int()?;
                let a = interp.pop_as_int()?;
                let prod = a * b;
                interp.push(Value::Int(prod % divisor));
                interp.push(Value::Int(prod / divisor));
                Ok(())
            },
            "( n1 n2 n3 -- n4 n5 ) Multiply n1 and n2, then divide by n3, push remainder and quotient.",
            "n1 n2 n3 -- n4 n5"
        );
    add_native_word!(
        interpreter,
        "1+",
        |interp: &mut dyn Interpreter| {
            let a = interp.pop_as_int()?;
            interp.push(Value::Int(a + 1));
            Ok(())
        },
        "( n -- n+1 ) Adds 1 to the top of the stack.",
        "( n -- n+1 )"
    );
    add_native_word!(
        interpreter,
        "1-",
        |interp: &mut dyn Interpreter| {
            let a = interp.pop_as_int()?;
            interp.push(Value::Int(a - 1));
            Ok(())
        },
        "( n -- n-1 ) Subtracts 1 from the top of the stack.",
        "( n -- n-1 )"
    );
    add_native_word!(
        interpreter,
        "2*",
        |interp: &mut dyn Interpreter| {
            let a = interp.pop_as_int()?;
            interp.push(Value::Int(a * 2));
            Ok(())
        },
        "( n -- n*2 ) Multiplies the top of the stack by 2.",
        "( n -- n*2 )"
    );
    add_native_word!(
        interpreter,
        "2/",
        |interp: &mut dyn Interpreter| {
            let a = interp.pop_as_int()?;
            interp.push(Value::Int(a / 2));
            Ok(())
        },
        "( n -- n/2 ) Divides the top of the stack by 2.",
        "( n -- n/2 )"
    );
    add_native_word!(
        interpreter,
        "mod",
        |interp: &mut dyn Interpreter| {
            let b = interp.pop_as_int()?;
            let a = interp.pop_as_int()?;
            interp.push(Value::Int(a % b));
            Ok(())
        },
        "( n1 n2 -- n ) Remainder after dividing n1 by n2.",
        "( n1 n2 -- n )"
    );
    add_native_word!(
        interpreter,
        "/mod",
        |interp: &mut dyn Interpreter| {
            let b = interp.pop_as_int()?;
            let a = interp.pop_as_int()?;
            interp.push(Value::Int(a % b));
            interp.push(Value::Int(a / b));
            Ok(())
        },
        "( n1 n2 -- rem quot ) Remainder and quotient after dividing n1 by n2.",
        "( n1 n2 -- rem quot )"
    );

    add_native_word!(
        interpreter,
        "abs",
        |interp: &mut dyn Interpreter| {
            let a = interp.pop_as_int()?;
            interp.push(Value::Int(a.abs()));
            Ok(())
        },
        "( n -- |n| ) Absolute value of the top of the stack.",
        "( n -- |n| )"
    );

    add_native_word!(
        interpreter,
        "negate",
        |interp: &mut dyn Interpreter| {
            let a = interp.pop_as_int()?;
            interp.push(Value::Int(-a));
            Ok(())
        },
        "( n -- -n ) Negates the top of the stack.",
        "( n -- -n )"
    );
}
