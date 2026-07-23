use cosmic_calc_plus::engine::engineering::EngineeringEngine;
use cosmic_calc_plus::engine::{AngleMode, Evaluate};

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn test_sin_deg() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("sin(90)").unwrap().value, 1.0));
}

#[test]
fn test_cos_rad() {
    let engine = EngineeringEngine::new(AngleMode::Rad);
    assert!(approx_eq(engine.evaluate("cos(0)").unwrap().value, 1.0));
}

#[test]
fn test_tan() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("tan(45)").unwrap().value, 1.0));
}

#[test]
fn test_asin() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("asin(1)").unwrap().value, 90.0));
}

#[test]
fn test_acos() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("acos(1)").unwrap().value, 0.0));
}

#[test]
fn test_atan() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("atan(1)").unwrap().value, 45.0));
}

#[test]
fn test_sin_grad() {
    let engine = EngineeringEngine::new(AngleMode::Grad);
    assert!(approx_eq(engine.evaluate("sin(100)").unwrap().value, 1.0));
}

#[test]
fn test_log() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("log(100)").unwrap().value, 2.0));
}

#[test]
fn test_ln() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("ln(1)").unwrap().value, 0.0));
}

#[test]
fn test_sqrt() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("sqrt(144)").unwrap().value, 12.0));
}

#[test]
fn test_factorial() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("fact(5)").unwrap().value, 120.0));
}

#[test]
fn test_abs() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("abs(-42)").unwrap().value, 42.0));
}

#[test]
fn test_floor_ceil() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("floor(3.7)").unwrap().value, 3.0));
    assert!(approx_eq(engine.evaluate("ceil(3.2)").unwrap().value, 4.0));
}

#[test]
fn test_constants() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(
        engine.evaluate("pi").unwrap().value,
        std::f64::consts::PI
    ));
}

#[test]
fn test_complex_expression() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    let result = engine.evaluate("2*sin(30)+1").unwrap().value;
    assert!(approx_eq(result, 2.0));
}

#[test]
fn test_alt_bases() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    let result = engine.evaluate("255").unwrap();
    let bases = result.alt_bases.unwrap();
    assert_eq!(bases.hex, "FF");
    assert_eq!(bases.oct, "377");
    assert_eq!(bases.bin, "11111111");
}

#[test]
fn test_sqrt_negative_domain_error() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("sqrt(-1)").is_err());
}

#[test]
fn test_log_zero_domain_error() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("log(0)").is_err());
}

#[test]
fn test_ln_negative_domain_error() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("ln(-5)").is_err());
}

#[test]
fn test_asin_out_of_range() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("asin(2)").is_err());
}

#[test]
fn test_acos_out_of_range() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("acos(-2)").is_err());
}

#[test]
fn test_factorial_negative() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("fact(-1)").is_err());
}

#[test]
fn test_factorial_non_integer() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("fact(3.5)").is_err());
}

#[test]
fn test_factorial_overflow() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("fact(171)").is_err());
}

#[test]
fn test_factorial_170_ok() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("fact(170)").is_ok());
}

// The engineering UI now wraps a pending operand as a function argument
// (e.g. `34` + cos -> `cos(34)`). These lock in that the shapes it generates
// parse and evaluate correctly, including the mid-expression form.
#[test]
fn test_wrapped_prefix_function() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("cos(0)").unwrap().value, 1.0));
}

#[test]
fn test_wrapped_prefix_function_mid_expression() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    // 2 + cos(0) = 3
    assert!(approx_eq(engine.evaluate("2+cos(0)").unwrap().value, 3.0));
}

#[test]
fn test_wrapped_prefix_function_negative_argument() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    // cos is even: cos(-90) == cos(90) == 0
    assert!(approx_eq(engine.evaluate("cos(-90)").unwrap().value, 0.0));
}

// The n! button appends "!"; the parser routes postfix "!" to fact().
#[test]
fn test_factorial_postfix() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("5!").unwrap().value, 120.0));
    assert!(approx_eq(engine.evaluate("0!").unwrap().value, 1.0));
}

#[test]
fn test_factorial_postfix_in_expression() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("3!+1").unwrap().value, 7.0));
}

#[test]
fn test_factorial_binds_tighter_than_power() {
    // 2 ^ 3! == 2 ^ 6 == 64 (factorial applies before exponent)
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("2^3!").unwrap().value, 64.0));
}

#[test]
fn test_factorial_postfix_domain_errors() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("3.5!").is_err()); // non-integer
    assert!(engine.evaluate("171!").is_err()); // overflow
}

// Alt-base representations backing the HEX/OCT/BIN buttons.
#[test]
fn test_alt_bases_integer() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    let r = engine.evaluate("255").unwrap();
    let bases = r.alt_bases.expect("integer result should have alt bases");
    assert_eq!(bases.hex, "FF");
    assert_eq!(bases.oct, "377");
    assert_eq!(bases.bin, "11111111");
}

#[test]
fn test_alt_bases_none_for_non_integer() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("3.14").unwrap().alt_bases.is_none());
}

#[test]
fn test_alt_bases_none_for_negative() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("0-5").unwrap().alt_bases.is_none());
}

// Bitwise ops flow through the engineering engine end-to-end.
#[test]
fn test_bitwise_through_engine() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(approx_eq(engine.evaluate("5 AND 3").unwrap().value, 1.0));
    assert!(approx_eq(engine.evaluate("NOT 5").unwrap().value, -6.0));
    assert!(approx_eq(engine.evaluate("1 << 4").unwrap().value, 16.0));
}

#[test]
fn test_bitwise_non_integer_errors_through_engine() {
    let engine = EngineeringEngine::new(AngleMode::Deg);
    assert!(engine.evaluate("2.5 OR 1").is_err());
}
