use crate::apexlang::ast::{BinaryOp, Expr, Program, Statement, UnaryOp, Value};
use crate::apexlang::error::ApexError;
use crate::apexlang::parser::Parser;

/// Parse and evaluate the provided ApexLang source code.
///
/// The interpreter implements the minimal MVP described in the design document:
/// a single `fn apex() { ... }` entry point containing a `return` statement with
/// arithmetic expressions over numeric literals. Integer and floating-point
/// literals are tracked separately so the evaluator can maintain integer
/// semantics, fall back to `Number` values on overflow, and honor `%` modulo
/// operations.
pub fn evaluate_source(source: &str) -> Result<Value, ApexError> {
    let program = Parser::parse(source)?;
    evaluate_program(&program)
}

fn evaluate_program(program: &Program) -> Result<Value, ApexError> {
    let apex_fn = program
        .functions
        .iter()
        .find(|f| f.name == "apex")
        .ok_or_else(|| ApexError::new("Expected an 'apex' entry-point function"))?;

    for statement in &apex_fn.body {
        match statement {
            Statement::Return(expr) => return evaluate_expr(expr),
        }
    }

    Err(ApexError::new("The 'apex' function must return a value"))
}

fn evaluate_expr(expr: &Expr) -> Result<Value, ApexError> {
    match expr {
        Expr::Literal(value) => Ok(value.clone()),
        Expr::Unary(op, value) => {
            let inner = evaluate_expr(value)?;
            match op {
                UnaryOp::Plus => Ok(inner),
                UnaryOp::Minus => match inner {
                    Value::Int(v) => match v.checked_neg() {
                        Some(value) => Ok(Value::Int(value)),
                        None => Ok(Value::Number(-(v as f64))),
                    },
                    Value::Number(v) => Ok(Value::Number(-v)),
                },
            }
        }
        Expr::Binary(lhs, op, rhs) => {
            let left = evaluate_expr(lhs)?;
            let right = evaluate_expr(rhs)?;
            apply_binary(op, left, right)
        }
    }
}

fn apply_binary(op: &BinaryOp, left: Value, right: Value) -> Result<Value, ApexError> {
    use BinaryOp::*;
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => match op {
            Add => match l.checked_add(r) {
                Some(value) => Ok(Value::Int(value)),
                None => Ok(Value::Number(l as f64 + r as f64)),
            },
            Sub => match l.checked_sub(r) {
                Some(value) => Ok(Value::Int(value)),
                None => Ok(Value::Number(l as f64 - r as f64)),
            },
            Mul => match l.checked_mul(r) {
                Some(value) => Ok(Value::Int(value)),
                None => Ok(Value::Number((l as f64) * (r as f64))),
            },
            Div => {
                if r == 0 {
                    Err(ApexError::new("Division by zero"))
                } else {
                    Ok(Value::Int(l / r))
                }
            }
            Mod => {
                if r == 0 {
                    Err(ApexError::new("Modulo by zero"))
                } else {
                    Ok(Value::Int(l % r))
                }
            }
        },
        (Value::Number(l), Value::Number(r)) => match op {
            Add => Ok(Value::Number(l + r)),
            Sub => Ok(Value::Number(l - r)),
            Mul => Ok(Value::Number(l * r)),
            Div => {
                if r == 0.0 {
                    Err(ApexError::new("Division by zero"))
                } else {
                    Ok(Value::Number(l / r))
                }
            }
            Mod => {
                if r == 0.0 {
                    Err(ApexError::new("Modulo by zero"))
                } else {
                    Ok(Value::Number(l % r))
                }
            }
        },
        (left, right) => {
            let l = left.as_f64();
            let r = right.as_f64();
            match op {
                Add => Ok(Value::Number(l + r)),
                Sub => Ok(Value::Number(l - r)),
                Mul => Ok(Value::Number(l * r)),
                Div => {
                    if r == 0.0 {
                        Err(ApexError::new("Division by zero"))
                    } else {
                        Ok(Value::Number(l / r))
                    }
                }
                Mod => {
                    if r == 0.0 {
                        Err(ApexError::new("Modulo by zero"))
                    } else {
                        Ok(Value::Number(l % r))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate_source;
    use crate::apexlang::ast::Value;

    #[test]
    fn evaluates_simple_expression() {
        let source = r#"
            fn apex() {
                return (1 + 2) * 3 - 4 / 2;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert_eq!(value, Value::Int(7));
    }

    #[test]
    fn supports_unary_operations() {
        let source = r#"
            fn apex() {
                return -(3 + 5) + +2;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert_eq!(value, Value::Int(-6));
    }

    #[test]
    fn widens_mixed_arithmetic() {
        let source = r#"
            fn apex() {
                return 4 / 2 + 1.5;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert!((value.as_f64() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn supports_modulo() {
        let source = r#"
            fn apex() {
                return 17 % 5;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert_eq!(value, Value::Int(2));
    }

    #[test]
    fn rejects_division_by_zero() {
        let source = r#"
            fn apex() {
                return 1 / 0;
            }
        "#;

        let error = evaluate_source(source).expect_err("division by zero");
        assert!(error.message().contains("zero"));
    }

    #[test]
    fn widens_on_integer_overflow() {
        let source = r#"
            fn apex() {
                return 9223372036854775807 + 1;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert!(matches!(value, Value::Number(_)));
    }

    #[test]
    fn reports_missing_apex_function() {
        let source = r#"
            fn other() {
                return 1;
            }
        "#;

        let error = evaluate_source(source).expect_err("missing apex function");
        assert!(error.message().contains("apex"));
    }

    #[test]
    fn reports_missing_return() {
        let source = r#"
            fn apex() {
                // No return here
            }
        "#;

        let error = evaluate_source(source).expect_err("missing return");
        assert!(error.message().contains("must return"));
    }
}
