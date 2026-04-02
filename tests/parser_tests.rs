use cosmic_ext_calculator::engine::parser::Parser;
use cosmic_ext_calculator::engine::CalcError;

#[test]
fn integer_literal() {
    let mut p = Parser::new();
    assert_eq!(p.parse("42").unwrap(), 42.0);
}

#[test]
fn decimal_literal() {
    let mut p = Parser::new();
    assert!((p.parse("3.14").unwrap() - 3.14).abs() < 1e-10);
}

#[test]
fn addition() {
    let mut p = Parser::new();
    assert_eq!(p.parse("2+3").unwrap(), 5.0);
}

#[test]
fn subtraction() {
    let mut p = Parser::new();
    assert_eq!(p.parse("10-3").unwrap(), 7.0);
}

#[test]
fn multiplication() {
    let mut p = Parser::new();
    assert_eq!(p.parse("4*5").unwrap(), 20.0);
}

#[test]
fn division() {
    let mut p = Parser::new();
    assert_eq!(p.parse("15/3").unwrap(), 5.0);
}

#[test]
fn division_by_zero() {
    let mut p = Parser::new();
    let result = p.parse("1/0");
    assert!(matches!(result, Err(CalcError::DivisionByZero)));
}

#[test]
fn operator_precedence_mul_over_add() {
    let mut p = Parser::new();
    assert_eq!(p.parse("2+3*4").unwrap(), 14.0);
}

#[test]
fn parentheses() {
    let mut p = Parser::new();
    assert_eq!(p.parse("(2+3)*4").unwrap(), 20.0);
}

#[test]
fn nested_parentheses() {
    let mut p = Parser::new();
    assert_eq!(p.parse("((2+3)*(4-1))").unwrap(), 15.0);
}

#[test]
fn unary_minus_standalone() {
    let mut p = Parser::new();
    assert_eq!(p.parse("-5").unwrap(), -5.0);
}

#[test]
fn unary_minus_in_expression() {
    let mut p = Parser::new();
    assert_eq!(p.parse("3+-2").unwrap(), 1.0);
}

#[test]
fn power_basic() {
    let mut p = Parser::new();
    assert_eq!(p.parse("2^3").unwrap(), 8.0);
}

#[test]
fn power_right_associative() {
    let mut p = Parser::new();
    // 2^3^2 = 2^(3^2) = 2^9 = 512
    assert_eq!(p.parse("2^3^2").unwrap(), 512.0);
}

#[test]
fn modulus() {
    let mut p = Parser::new();
    assert_eq!(p.parse("17%5").unwrap(), 2.0);
}

#[test]
fn postfix_percentage() {
    let mut p = Parser::new();
    assert!((p.parse("50%").unwrap() - 0.5).abs() < 1e-10);
}

#[test]
fn implicit_multiplication_number_paren() {
    let mut p = Parser::new();
    assert_eq!(p.parse("3(4+1)").unwrap(), 15.0);
}

#[test]
fn complex_expression() {
    let mut p = Parser::new();
    assert_eq!(p.parse("(3+4)*2-1").unwrap(), 13.0);
}

#[test]
fn invalid_expression() {
    let mut p = Parser::new();
    let result = p.parse("2++*3");
    assert!(matches!(result, Err(CalcError::InvalidExpression(_))));
}

#[test]
fn empty_expression() {
    let mut p = Parser::new();
    let result = p.parse("");
    assert!(matches!(result, Err(CalcError::InvalidExpression(_))));
}

#[test]
fn constant_pi() {
    let mut p = Parser::new();
    assert!((p.parse("pi").unwrap() - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn constant_pi_unicode() {
    let mut p = Parser::new();
    assert!((p.parse("π").unwrap() - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn constant_e() {
    let mut p = Parser::new();
    assert!((p.parse("e").unwrap() - std::f64::consts::E).abs() < 1e-10);
}

#[test]
fn implicit_multiplication_with_constant() {
    let mut p = Parser::new();
    assert!((p.parse("2pi").unwrap() - 2.0 * std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn unicode_multiply() {
    let mut p = Parser::new();
    assert_eq!(p.parse("3×4").unwrap(), 12.0);
}

#[test]
fn unicode_divide() {
    let mut p = Parser::new();
    assert_eq!(p.parse("12÷3").unwrap(), 4.0);
}

#[test]
fn function_call() {
    let mut p = Parser::new();
    let result = p.parse_with_functions("sqrt(9)", |name, args| {
        match name {
            "sqrt" => Ok(f64::sqrt(args[0])),
            _ => Err(CalcError::InvalidExpression(format!("Unknown function: {name}"))),
        }
    });
    assert_eq!(result.unwrap(), 3.0);
}

#[test]
fn function_call_multi_arg() {
    let mut p = Parser::new();
    let result = p.parse_with_functions("max(3,7)", |name, args| {
        match name {
            "max" => Ok(args[0].max(args[1])),
            _ => Err(CalcError::InvalidExpression(format!("Unknown function: {name}"))),
        }
    });
    assert_eq!(result.unwrap(), 7.0);
}

#[test]
fn whitespace_handling() {
    let mut p = Parser::new();
    assert_eq!(p.parse(" 2 + 3 ").unwrap(), 5.0);
}

#[test]
fn percentage_in_expression() {
    let mut p = Parser::new();
    // 50% + 1 = 0.5 + 1 = 1.5
    assert!((p.parse("50%+1").unwrap() - 1.5).abs() < 1e-10);
}
