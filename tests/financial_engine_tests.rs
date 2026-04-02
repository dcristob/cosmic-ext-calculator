use cosmic_ext_calculator::engine::financial::{FinancialEngine, TvmParams, TvmSolveFor};

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
fn test_margin() {
    let engine = FinancialEngine;
    assert!(approx_eq(engine.margin(80.0, 100.0), 20.0, 0.01));
}

#[test]
fn test_markup() {
    let engine = FinancialEngine;
    assert!(approx_eq(engine.markup(80.0, 100.0), 25.0, 0.01));
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
