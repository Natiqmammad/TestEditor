use crate::apexlang::ast::{BinaryOp, Expr, Program, Statement, UnaryOp};
use crate::apexlang::error::ApexError;
use crate::apexlang::parser::Parser;

/// Parse and evaluate the provided ApexLang source code.
///
/// The interpreter implements the minimal MVP described in the design document:
/// a single `fn apex() { ... }` entry point containing a `return` statement with
/// arithmetic expressions over numeric literals.
pub fn evaluate_source(source: &str) -> Result<f64, ApexError> {
    let program = Parser::parse(source)?;
    evaluate_program(&program)
}

fn evaluate_program(program: &Program) -> Result<f64, ApexError> {
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

fn evaluate_expr(expr: &Expr) -> Result<f64, ApexError> {
    match expr {
        Expr::Number(value) => Ok(*value),
        Expr::Unary(op, value) => {
            let inner = evaluate_expr(value)?;
            Ok(match op {
                UnaryOp::Plus => inner,
                UnaryOp::Minus => -inner,
            })
        }
        Expr::Binary(lhs, op, rhs) => {
            let left = evaluate_expr(lhs)?;
            let right = evaluate_expr(rhs)?;
            Ok(match op {
                BinaryOp::Add => left + right,
                BinaryOp::Sub => left - right,
                BinaryOp::Mul => left * right,
                BinaryOp::Div => left / right,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate_source;

    #[test]
    fn evaluates_simple_expression() {
        let source = r#"
            fn apex() {
                return (1 + 2) * 3 - 4 / 2;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert!((value - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn supports_unary_operations() {
        let source = r#"
            fn apex() {
                return -(3 + 5) + +2;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert!((value + 6.0).abs() < f64::EPSILON);
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
