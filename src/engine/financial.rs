use super::CalcError;

pub struct FinancialEngine;

pub struct TvmParams {
    pub n: Option<f64>,
    pub rate: Option<f64>,
    pub pv: Option<f64>,
    pub pmt: Option<f64>,
    pub fv: Option<f64>,
}

pub enum TvmSolveFor {
    N,
    Rate,
    Pv,
    Pmt,
    Fv,
}

impl FinancialEngine {
    pub fn solve_tvm(&self, p: TvmParams, solve_for: TvmSolveFor) -> Result<f64, CalcError> {
        match solve_for {
            TvmSolveFor::Fv => {
                let n =
                    p.n.ok_or_else(|| CalcError::InvalidExpression("missing N".into()))?;
                let rate = p
                    .rate
                    .ok_or_else(|| CalcError::InvalidExpression("missing rate".into()))?;
                let pv =
                    p.pv.ok_or_else(|| CalcError::InvalidExpression("missing PV".into()))?;
                let pmt = p
                    .pmt
                    .ok_or_else(|| CalcError::InvalidExpression("missing PMT".into()))?;
                let i = rate / 100.0;
                if i == 0.0 {
                    Ok(-(pv + pmt * n))
                } else {
                    let compound = (1.0 + i).powf(n);
                    Ok(-(pv * compound + pmt * (compound - 1.0) / i))
                }
            }
            TvmSolveFor::Pv => {
                let n =
                    p.n.ok_or_else(|| CalcError::InvalidExpression("missing N".into()))?;
                let rate = p
                    .rate
                    .ok_or_else(|| CalcError::InvalidExpression("missing rate".into()))?;
                let pmt = p
                    .pmt
                    .ok_or_else(|| CalcError::InvalidExpression("missing PMT".into()))?;
                let fv =
                    p.fv.ok_or_else(|| CalcError::InvalidExpression("missing FV".into()))?;
                let i = rate / 100.0;
                if i == 0.0 {
                    Ok(-(fv + pmt * n))
                } else {
                    let compound = (1.0 + i).powf(n);
                    Ok(-(fv + pmt * (compound - 1.0) / i) / compound)
                }
            }
            TvmSolveFor::Pmt => {
                let n =
                    p.n.ok_or_else(|| CalcError::InvalidExpression("missing N".into()))?;
                let rate = p
                    .rate
                    .ok_or_else(|| CalcError::InvalidExpression("missing rate".into()))?;
                let pv =
                    p.pv.ok_or_else(|| CalcError::InvalidExpression("missing PV".into()))?;
                let fv =
                    p.fv.ok_or_else(|| CalcError::InvalidExpression("missing FV".into()))?;
                let i = rate / 100.0;
                if i == 0.0 {
                    if n == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    Ok(-(pv + fv) / n)
                } else {
                    let compound = (1.0 + i).powf(n);
                    Ok(-(pv * compound + fv) / ((compound - 1.0) / i))
                }
            }
            TvmSolveFor::N => {
                let rate = p
                    .rate
                    .ok_or_else(|| CalcError::InvalidExpression("missing rate".into()))?;
                let pv =
                    p.pv.ok_or_else(|| CalcError::InvalidExpression("missing PV".into()))?;
                let pmt = p
                    .pmt
                    .ok_or_else(|| CalcError::InvalidExpression("missing PMT".into()))?;
                let fv =
                    p.fv.ok_or_else(|| CalcError::InvalidExpression("missing FV".into()))?;
                let i = rate / 100.0;
                if i == 0.0 {
                    if pmt == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    Ok(-(pv + fv) / pmt)
                } else {
                    let numerator = -fv * i + pmt;
                    let denominator = pv * i + pmt;
                    if denominator == 0.0 || numerator == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    let ratio = numerator / denominator;
                    if ratio <= 0.0 {
                        return Err(CalcError::DomainError(
                            "invalid parameters for N solve".into(),
                        ));
                    }
                    let result = ratio.ln() / (1.0 + i).ln();
                    if result.is_nan() || result.is_infinite() {
                        return Err(CalcError::DomainError("invalid result for N".into()));
                    }
                    Ok(result)
                }
            }
            TvmSolveFor::Rate => {
                let n =
                    p.n.ok_or_else(|| CalcError::InvalidExpression("missing N".into()))?;
                let pv =
                    p.pv.ok_or_else(|| CalcError::InvalidExpression("missing PV".into()))?;
                let pmt = p
                    .pmt
                    .ok_or_else(|| CalcError::InvalidExpression("missing PMT".into()))?;
                let fv =
                    p.fv.ok_or_else(|| CalcError::InvalidExpression("missing FV".into()))?;

                let mut i = 0.1_f64;
                let max_iter = 1000;
                let tol = 1e-10;

                for _ in 0..max_iter {
                    let compound = (1.0 + i).powf(n);
                    let f = fv + pv * compound + pmt * (compound - 1.0) / i;
                    let compound_d = (1.0 + i).powf(n - 1.0);
                    let f_prime = pv * n * compound_d
                        + pmt * (n * compound_d * i - (compound - 1.0)) / (i * i);

                    if f_prime.abs() < 1e-20 {
                        return Err(CalcError::ConvergenceError);
                    }

                    let i_new = i - f / f_prime;
                    if (i_new - i).abs() < tol {
                        return Ok(i_new * 100.0);
                    }
                    i = i_new;
                }

                Err(CalcError::ConvergenceError)
            }
        }
    }

    /// Selling price for a cost and a target margin (percent of the price).
    /// `price = cost / (1 - margin/100)`.
    pub fn price_from_margin(&self, cost: f64, margin_pct: f64) -> Result<f64, CalcError> {
        let denom = 1.0 - margin_pct / 100.0;
        if denom <= 0.0 {
            return Err(CalcError::DomainError("margin must be below 100%".into()));
        }
        Ok(cost / denom)
    }

    /// Selling price for a cost and a markup (percent of the cost).
    /// `price = cost * (1 + markup/100)`.
    pub fn price_from_markup(&self, cost: f64, markup_pct: f64) -> Result<f64, CalcError> {
        let factor = 1.0 + markup_pct / 100.0;
        if factor <= 0.0 {
            return Err(CalcError::DomainError("markup must be above -100%".into()));
        }
        Ok(cost * factor)
    }

    pub fn add_tax(&self, net: f64, tax_rate: f64) -> f64 {
        net * (1.0 + tax_rate / 100.0)
    }

    pub fn remove_tax(&self, gross: f64, tax_rate: f64) -> f64 {
        gross / (1.0 + tax_rate / 100.0)
    }
}
