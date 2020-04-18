use serde_json::Value;

mod data;
mod error;
// TODO consider whether this should be public; move doctests if so
pub mod js_op;
mod op;
mod value;

use error::Error;
use value::{Evaluated, Parsed};

const NULL: Value = Value::Null;

trait Parser<'a>: Sized + Into<Value> {
    fn from_value(value: &'a Value) -> Result<Option<Self>, Error>;
    fn evaluate(&self, data: &'a Value) -> Result<Evaluated, Error>;
}

/// Run JSONLogic for the given operation and data.
///
pub fn jsonlogic(value: &Value, data: &Value) -> Result<Value, Error> {
    let parsed = Parsed::from_value(&value)?;
    parsed.evaluate(data).map(Value::from)
}

#[cfg(test)]
mod jsonlogic_tests {
    use super::*;
    use serde_json::json;

    fn no_op_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            // Passing a static value returns the value unchanged.
            (json!("foo"), json!({}), Ok(json!("foo"))),
            (json!(""), json!({}), Ok(json!(""))),
            (json!([1, 2]), json!({}), Ok(json!([1, 2]))),
            (json!([]), json!({}), Ok(json!([]))),
            (json!(null), json!({}), Ok(json!(null))),
            (json!(0), json!({}), Ok(json!(0))),
            (json!(234), json!({}), Ok(json!(234))),
            (json!({}), json!({}), Ok(json!({}))),
            // Note: as of this writing, this behavior differs from the
            // original jsonlogic implementation, which errors for objects of
            // length one, due to attempting to parse their key as an operation
            (json!({"a": 1}), json!({}), Ok(json!({"a": 1}))),
            (
                json!({"a": 1, "b": 2}),
                json!({}),
                Ok(json!({"a": 1, "b": 2})),
            ),
        ]
    }

    fn abstract_eq_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (json!({"==": [1, 1]}), json!({}), Ok(json!(true))),
            (json!({"==": [1, 2]}), json!({}), Ok(json!(false))),
            (json!({"==": [1, "1"]}), json!({}), Ok(json!(true))),
            (
                json!({"==": [{}, "[object Object]"]}),
                json!({}),
                Ok(json!(true)),
            ),
            (json!({"==": [1, [1]]}), json!({}), Ok(json!(true))),
            (json!({"==": [1, true]}), json!({}), Ok(json!(true))),
            // Recursive evaluation
            (
                json!({"==": [true, {"==": [1, 1]}]}),
                json!({}),
                Ok(json!(true)),
            ),
            (
                json!({"==": [{"==": [{"==": [1, 1]}, true]}, {"==": [1, 1]}]}),
                json!({}),
                Ok(json!(true)),
            ),
            // Wrong number of arguments
            (json!({"==": [1]}), json!({}), Err(())),
            (json!({"==": [1, 1, 1]}), json!({}), Err(())),
        ]
    }

    fn abstract_ne_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (json!({"!=": [1, 1]}), json!({}), Ok(json!(false))),
            (json!({"!=": [1, 2]}), json!({}), Ok(json!(true))),
            (json!({"!=": [1, "1"]}), json!({}), Ok(json!(false))),
            (
                json!({"!=": [{}, "[object Object]"]}),
                json!({}),
                Ok(json!(false)),
            ),
            (
                json!({"!=": [{"!=": [1, 2]}, 1]}),
                json!({}),
                Ok(json!(false)),
            ),
            // Wrong number of arguments
            (json!({"!=": [1]}), json!({}), Err(())),
            (json!({"!=": [1, 1, 1]}), json!({}), Err(())),
        ]
    }

    fn strict_eq_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (json!({"===": [1, 1]}), json!({}), Ok(json!(true))),
            (json!({"===": [1, 2]}), json!({}), Ok(json!(false))),
            (json!({"===": [1, "1"]}), json!({}), Ok(json!(false))),
            (
                json!({"===": [{}, "[object Object]"]}),
                json!({}),
                Ok(json!(false)),
            ),
            (json!({"===": [1, [1]]}), json!({}), Ok(json!(false))),
            (json!({"===": [1, true]}), json!({}), Ok(json!(false))),
            // Recursive evaluation
            (
                json!({"===": [true, {"===": [1, 1]}]}),
                json!({}),
                Ok(json!(true)),
            ),
            (
                json!({"===": [{"===": [{"===": [1, 1]}, true]}, {"===": [1, 1]}]}),
                json!({}),
                Ok(json!(true)),
            ),
            // Wrong number of arguments
            (json!({"===": [1]}), json!({}), Err(())),
            (json!({"===": [1, 1, 1]}), json!({}), Err(())),
        ]
    }

    fn strict_ne_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (json!({"!==": [1, 1]}), json!({}), Ok(json!(false))),
            (json!({"!==": [1, 2]}), json!({}), Ok(json!(true))),
            (json!({"!==": [1, "1"]}), json!({}), Ok(json!(true))),
            (
                json!({"!==": [{}, "[object Object]"]}),
                json!({}),
                Ok(json!(true)),
            ),
            (json!({"!==": [1, [1]]}), json!({}), Ok(json!(true))),
            (json!({"!==": [1, true]}), json!({}), Ok(json!(true))),
            // Recursive evaluation
            (
                json!({"!==": [true, {"!==": [1, 1]}]}),
                json!({}),
                Ok(json!(true)),
            ),
            (
                json!({"!==": [{"!==": [{"!==": [1, 1]}, false]}, {"!==": [1, 1]}]}),
                json!({}),
                Ok(json!(false)),
            ),
            // Wrong number of arguments
            (json!({"!==": [1]}), json!({}), Err(())),
            (json!({"!==": [1, 1, 1]}), json!({}), Err(())),
        ]
    }

    fn var_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            // Variable substitution
            (
                json!({"var": "foo"}),
                json!({"foo": "bar"}),
                Ok(json!("bar")),
            ),
            // Index into array data
            (json!({"var": 1}), json!(["foo", "bar"]), Ok(json!("bar"))),
            // Absent variable
            (json!({"var": "foo"}), json!({}), Ok(json!(null))),
            (
                json!({"==": [{"var": "first"}, true]}),
                json!({"first": true}),
                Ok(json!(true)),
            ),
            // Dotted variable substitution
            (
                json!({"var": "foo.bar"}),
                json!({"foo": {"bar": "baz"}}),
                Ok(json!("baz")),
            ),
            // Dotted variable with nested array access
            (
                json!({"var": "foo.1"}),
                json!({"foo": ["bar", "baz", "pop"]}),
                Ok(json!("baz")),
            ),
            // Absent dotted variable
            (
                json!({"var": "foo.bar"}),
                json!({"foo": {"baz": "baz"}}),
                Ok(json!(null)),
            ),
            // Non-object type in dotted variable path
            (
                json!({"var": "foo.bar.baz"}),
                json!({"foo": {"bar": 1}}),
                Ok(json!(null)),
            ),
            (
                json!({"var": "foo.bar"}),
                json!({"foo": "not an object"}),
                Ok(json!(null)),
            ),
        ]
    }

    fn missing_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            // "missing" data operator
            (
                json!({"missing": ["a", "b"]}),
                json!({"a": 1, "b": 2}),
                Ok(json!([])),
            ),
            (
                json!({"missing": ["a", "b"]}),
                json!({"a": 1}),
                Ok(json!(["b"])),
            ),
            (json!({"missing": [1, 5]}), json!([1, 2, 3]), Ok(json!([5]))),
        ]
    }

    fn missing_some_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            // "missing_some" data operator
            (
                json!({"missing_some": [1, ["a", "b"]]}),
                json!({"a": 1, "b": 2}),
                Ok(json!([])),
            ),
            (
                json!({"missing_some": [1, ["a", "b", "c"]]}),
                json!({"a": 1, "b": 2}),
                Ok(json!([])),
            ),
            (
                json!({"missing_some": [2, ["a", "b", "c"]]}),
                json!({"a": 1}),
                Ok(json!(["b", "c"])),
            ),
        ]
    }

    fn if_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (
                json!({"if": [true, "true", "false"]}),
                json!({}),
                Ok(json!("true")),
            ),
            (
                json!({"if": [false, "true", "false"]}),
                json!({}),
                Ok(json!("false")),
            ),
            (
                json!({"if": [false, "true", true, "true2"]}),
                json!({}),
                Ok(json!("true2")),
            ),
            (
                json!({"if": [false, "true", false, "true2", "false2"]}),
                json!({}),
                Ok(json!("false2")),
            ),
            (
                json!({"if": [{"===": [1, 1]}, "true", "false"]}),
                json!({}),
                Ok(json!("true")),
            ),
            (
                json!({"if": [{"===": [1, 2]}, "true", "false"]}),
                json!({}),
                Ok(json!("false")),
            ),
            (
                json!({"if": [{"===": [1, 2]}, "true", {"===": [1, 1]}, "true2"]}),
                json!({}),
                Ok(json!("true2")),
            ),
            (
                json!({"if": [{"===": [1, 2]}, "true", {"===": [1, 2]}, "true2", "false2"]}),
                json!({}),
                Ok(json!("false2")),
            ),
        ]
    }

    fn or_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (json!({"or": [true]}), json!({}), Ok(json!(true))),
            (json!({"or": [false]}), json!({}), Ok(json!(false))),
            (json!({"or": [false, true]}), json!({}), Ok(json!(true))),
            (
                json!({"or": [false, true, false]}),
                json!({}),
                Ok(json!(true)),
            ),
            (json!({"or": [false, false, 12]}), json!({}), Ok(json!(12))),
            (
                json!({"or": [false, false, 12, 13, 14]}),
                json!({}),
                Ok(json!(12)),
            ),
            (
                json!({"or": [false, false, 0, 12]}),
                json!({}),
                Ok(json!(12)),
            ),
            (
                json!({"or": [false, {"===": [1, 1]}]}),
                json!({}),
                Ok(json!(true)),
            ),
            (
                json!({"or": [false, {"===": [{"var": "foo"}, 1]}]}),
                json!({"foo": 1}),
                Ok(json!(true)),
            ),
            (
                json!({"or": [false, {"===": [{"var": "foo"}, 1]}]}),
                json!({"foo": 2}),
                Ok(json!(false)),
            ),
        ]
    }

    fn and_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (json!({"and": [true]}), json!({}), Ok(json!(true))),
            (json!({"and": [false]}), json!({}), Ok(json!(false))),
            (json!({"and": [false, true]}), json!({}), Ok(json!(false))),
            (json!({"and": [true, false]}), json!({}), Ok(json!(false))),
            (json!({"and": [true, true]}), json!({}), Ok(json!(true))),
            (
                json!({"and": [false, true, false]}),
                json!({}),
                Ok(json!(false)),
            ),
            (json!({"and": [12, true, 0]}), json!({}), Ok(json!(0))),
            (
                json!({"and": [12, true, 0, 12, false]}),
                json!({}),
                Ok(json!(0)),
            ),
            (json!({"and": [true, true, 12]}), json!({}), Ok(json!(12))),
            (
                json!({"and": [{"===": [1, 1]}, false]}),
                json!({}),
                Ok(json!(false)),
            ),
            (
                json!({"and": [{"===": [{"var": "foo"}, 1]}, true]}),
                json!({"foo": 1}),
                Ok(json!(true)),
            ),
            (
                json!({"and": [{"===": [{"var": "foo"}, 1]}, true]}),
                json!({"foo": 2}),
                Ok(json!(false)),
            ),
        ]
    }

    fn lt_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (json!({"<": [1, 2]}), json!({}), Ok(json!(true))),
            (json!({"<": [3, 2]}), json!({}), Ok(json!(false))),
            (
                json!({"<": [1, {"var": "foo"}]}),
                json!({"foo": 5}),
                Ok(json!(true)),
            ),
            (json!({"<": [1, 2, 3]}), json!({}), Ok(json!(true))),
            (json!({"<": [3, 2, 3]}), json!({}), Ok(json!(false))),
            (json!({"<": [1, 2, 1]}), json!({}), Ok(json!(false))),
        ]
    }

    fn plus_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (json!({"+": []}), json!({}), Ok(json!(0.0))),
            (json!({"+": [1]}), json!({}), Ok(json!(1.0))),
            (json!({"+": [1, 1]}), json!({}), Ok(json!(2.0))),
            (json!({"+": [1, 1, 1]}), json!({}), Ok(json!(3.0))),
            (json!({"+": [1, 1, false]}), json!({}), Err(())),
            (json!({"+": [1, 1, "1"]}), json!({}), Ok(json!(3.0))),
            (
                json!({"+": [1, 1, "123abc"]}), // WHY???
                json!({}),
                Ok(json!(125.0)),
            ),
        ]
    }

    fn bang_cases() -> Vec<(Value, Value, Result<Value, ()>)> {
        vec![
            (json!( {"!": []} ), json!({}), Err(())),
            (json!( {"!": [1, 2]} ), json!({}), Err(())),
            (json!({"!": [true]}), json!({}), Ok(json!(false))),
            (json!({"!": [1]}), json!({}), Ok(json!(false))),
            (json!({"!": [0]}), json!({}), Ok(json!(true))),
            (json!({"!": [[]]}), json!({}), Ok(json!(true))),
            (json!({"!": [{}]}), json!({}), Ok(json!(false))),
            (json!({"!": [""]}), json!({}), Ok(json!(true))),
            (json!({"!": ["foo"]}), json!({}), Ok(json!(false))),
            (json!({"!": true}), json!({}), Ok(json!(false))),
        ]
    }

    fn assert_jsonlogic((op, data, exp): (Value, Value, Result<Value, ()>)) -> () {
        println!("Running rule: {:?} with data: {:?}", op, data);
        let result = jsonlogic(&op, &data);
        println!("- Result: {:?}", result);
        println!("- Expected: {:?}", exp);
        if exp.is_ok() {
            assert_eq!(result.unwrap(), exp.unwrap());
        } else {
            result.unwrap_err();
        }
    }

    #[test]
    fn test_no_op() {
        no_op_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_abstract_eq_op() {
        abstract_eq_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_abstract_ne_op() {
        abstract_ne_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_strict_eq_op() {
        strict_eq_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_strict_ne_op() {
        strict_ne_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_var_data_op() {
        var_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_missing_data_op() {
        missing_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_missing_some_data_op() {
        missing_some_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_if_op() {
        if_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_or_op() {
        or_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_and_op() {
        and_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_lt_op() {
        lt_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_lte_op() {
        lt_cases()
            .into_iter()
            .map(|(op, data, exp)| {
                (
                    match op {
                        Value::Object(obj) => json!({"<=": obj.get("<").unwrap()}),
                        _ => panic!("bad operator"),
                    },
                    data,
                    exp,
                )
            })
            .for_each(assert_jsonlogic);
        abstract_eq_cases()
            .into_iter()
            // Only get cases that are equal, since we don't know whether
            // non-equality cases were lt or gt or what.
            .filter(|(_, _, exp)| match exp {
                Ok(Value::Bool(exp)) => *exp,
                _ => false,
            })
            .map(|(op, data, exp)| {
                (
                    match op {
                        Value::Object(obj) => json!({"<=": obj.get("==").unwrap()}),
                        _ => panic!("bad operator"),
                    },
                    data,
                    exp,
                )
            })
            .for_each(assert_jsonlogic);
    }

    #[test]
    fn test_plus_op() {
        plus_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_bang_op() {
        bang_cases().into_iter().for_each(assert_jsonlogic)
    }

    #[test]
    fn test_bang_bang_op() {
        // just assert the opposite for all the bang cases
        bang_cases()
            .into_iter()
            .map(|(op, data, exp)| {
                (
                    match op {
                        Value::Object(obj) => {
                            let args = obj.get("!").unwrap();
                            json!({ "!!": args })
                        }
                        _ => panic!("op not operator"),
                    },
                    data,
                    match exp {
                        Err(_) => exp,
                        Ok(exp) => match exp {
                            Value::Bool(exp) => Ok(Value::Bool(!exp)),
                            _ => panic!("unexpected expected"),
                        },
                    },
                )
            })
            .for_each(assert_jsonlogic)
    }
}
