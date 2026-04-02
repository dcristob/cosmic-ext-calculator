use cosmic_ext_calculator::engine::{Evaluate, standard::StandardEngine};

#[test]
fn test_basic_arithmetic() {
    let engine = StandardEngine;
    assert_eq!(engine.evaluate("2+3").unwrap().value, 5.0);
    assert_eq!(engine.evaluate("10-4").unwrap().value, 6.0);
    assert_eq!(engine.evaluate("3*7").unwrap().value, 21.0);
    assert_eq!(engine.evaluate("15/3").unwrap().value, 5.0);
}

#[test]
fn test_precedence() {
    let engine = StandardEngine;
    assert_eq!(engine.evaluate("2+3*4").unwrap().value, 14.0);
    assert_eq!(engine.evaluate("(2+3)*4").unwrap().value, 20.0);
}

#[test]
fn test_percentage() {
    let engine = StandardEngine;
    assert_eq!(engine.evaluate("50%").unwrap().value, 0.5);
}

#[test]
fn test_modulus() {
    let engine = StandardEngine;
    assert_eq!(engine.evaluate("17%5").unwrap().value, 2.0);
}

#[test]
fn test_division_by_zero() {
    let engine = StandardEngine;
    assert!(engine.evaluate("1/0").is_err());
}

#[test]
fn test_display_formatting() {
    let engine = StandardEngine;
    let result = engine.evaluate("1/3").unwrap();
    assert!(result.display.contains("0.333"));
    assert!(result.alt_bases.is_none());
}

#[test]
fn test_integer_display() {
    let engine = StandardEngine;
    let result = engine.evaluate("2+3").unwrap();
    assert_eq!(result.display, "5");
}
