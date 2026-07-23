use cosmic_calc_plus::engine::financial::{FinancialEngine, TvmParams, TvmSolveFor};

fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

#[test]
fn test_solve_fv() {
    let engine = FinancialEngine;
    let params = TvmParams {
        n: Some(10.0),
        rate: Some(5.0),
        pv: Some(-1000.0),
        pmt: Some(0.0),
        fv: None,
    };
    let result = engine.solve_tvm(params, TvmSolveFor::Fv).unwrap();
    assert!(approx_eq(result, 1628.89, 0.01));
}

#[test]
fn test_solve_pv() {
    let engine = FinancialEngine;
    let params = TvmParams {
        n: Some(5.0),
        rate: Some(6.0),
        pv: None,
        pmt: Some(0.0),
        fv: Some(10000.0),
    };
    let result = engine.solve_tvm(params, TvmSolveFor::Pv).unwrap();
    assert!(approx_eq(result, -7472.58, 0.01));
}

#[test]
fn test_solve_pmt() {
    // 30-year mortgage, $200000, 4% annual (monthly: 360 periods, 0.333% rate)
    let engine = FinancialEngine;
    let params = TvmParams {
        n: Some(360.0),
        rate: Some(4.0 / 12.0),
        pv: Some(200000.0),
        pmt: None,
        fv: Some(0.0),
    };
    let result = engine.solve_tvm(params, TvmSolveFor::Pmt).unwrap();
    assert!(approx_eq(result, -954.83, 0.01));
}

#[test]
fn test_solve_n() {
    let engine = FinancialEngine;
    let params = TvmParams {
        n: None,
        rate: Some(7.0),
        pv: Some(-1000.0),
        pmt: Some(0.0),
        fv: Some(2000.0),
    };
    let result = engine.solve_tvm(params, TvmSolveFor::N).unwrap();
    assert!(approx_eq(result, 10.245, 0.01));
}

#[test]
fn test_solve_rate() {
    let engine = FinancialEngine;
    let params = TvmParams {
        n: Some(10.0),
        rate: None,
        pv: Some(-1000.0),
        pmt: Some(0.0),
        fv: Some(2000.0),
    };
    let result = engine.solve_tvm(params, TvmSolveFor::Rate).unwrap();
    assert!(approx_eq(result, 7.177, 0.01));
}

#[test]
fn test_price_from_margin() {
    let engine = FinancialEngine;
    // cost 60, 40% margin -> 60 / (1 - 0.40) = 100
    assert!(approx_eq(
        engine.price_from_margin(60.0, 40.0).unwrap(),
        100.0,
        1e-9
    ));
}

#[test]
fn test_price_from_markup() {
    let engine = FinancialEngine;
    // cost 60, 66.6667% markup -> 60 * (1 + 0.666667) = 100
    assert!(approx_eq(
        engine.price_from_markup(60.0, 66.666_667).unwrap(),
        100.0,
        1e-4
    ));
}

#[test]
fn test_tax_add() {
    let engine = FinancialEngine;
    assert!(approx_eq(engine.add_tax(100.0, 21.0), 121.0, 0.01));
}

#[test]
fn test_tax_remove() {
    let engine = FinancialEngine;
    assert!(approx_eq(engine.remove_tax(121.0, 21.0), 100.0, 0.01));
}

#[test]
fn test_price_from_margin_100pct_errors() {
    // A 100% margin has no valid price (divide by zero).
    let engine = FinancialEngine;
    assert!(engine.price_from_margin(60.0, 100.0).is_err());
}

#[test]
fn test_price_from_markup_below_neg100_errors() {
    // A markup below -100% would produce a negative price.
    let engine = FinancialEngine;
    assert!(engine.price_from_markup(60.0, -150.0).is_err());
}
