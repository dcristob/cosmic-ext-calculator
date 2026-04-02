use std::f64::consts::PI;

use super::parser::Parser;
use super::{AltBases, AngleMode, CalcError, CalcResult, Evaluate};

pub struct EngineeringEngine {
    angle_mode: AngleMode,
}

impl EngineeringEngine {
    pub fn new(angle_mode: AngleMode) -> Self {
        Self { angle_mode }
    }

    /// Convert from the current angle mode to radians.
    fn to_radians(&self, value: f64) -> f64 {
        match self.angle_mode {
            AngleMode::Deg => value.to_radians(),
            AngleMode::Rad => value,
            AngleMode::Grad => value * PI / 200.0,
        }
    }

    /// Convert from radians to the current angle mode.
    fn from_radians(&self, value: f64) -> f64 {
        match self.angle_mode {
            AngleMode::Deg => value.to_degrees(),
            AngleMode::Rad => value,
            AngleMode::Grad => value * 200.0 / PI,
        }
    }

    fn eval_function(&self, name: &str, args: &[f64]) -> Result<f64, CalcError> {
        let expect_one = |args: &[f64]| -> Result<f64, CalcError> {
            if args.len() != 1 {
                return Err(CalcError::InvalidExpression(format!(
                    "{name} expects 1 argument, got {}",
                    args.len()
                )));
            }
            Ok(args[0])
        };

        match name {
            // Trigonometric
            "sin" => {
                let v = expect_one(args)?;
                Ok(self.to_radians(v).sin())
            }
            "cos" => {
                let v = expect_one(args)?;
                Ok(self.to_radians(v).cos())
            }
            "tan" => {
                let v = expect_one(args)?;
                Ok(self.to_radians(v).tan())
            }
            "asin" => {
                let v = expect_one(args)?;
                if v < -1.0 || v > 1.0 {
                    return Err(CalcError::DomainError(
                        "asin argument must be in [-1, 1]".into(),
                    ));
                }
                Ok(self.from_radians(v.asin()))
            }
            "acos" => {
                let v = expect_one(args)?;
                if v < -1.0 || v > 1.0 {
                    return Err(CalcError::DomainError(
                        "acos argument must be in [-1, 1]".into(),
                    ));
                }
                Ok(self.from_radians(v.acos()))
            }
            "atan" => {
                let v = expect_one(args)?;
                Ok(self.from_radians(v.atan()))
            }

            // Logarithmic
            "log" => {
                let v = expect_one(args)?;
                if v <= 0.0 {
                    return Err(CalcError::DomainError(
                        "log argument must be positive".into(),
                    ));
                }
                Ok(v.log10())
            }
            "ln" => {
                let v = expect_one(args)?;
                if v <= 0.0 {
                    return Err(CalcError::DomainError(
                        "ln argument must be positive".into(),
                    ));
                }
                Ok(v.ln())
            }

            // Power/roots
            "sqrt" => {
                let v = expect_one(args)?;
                if v < 0.0 {
                    return Err(CalcError::DomainError(
                        "sqrt of negative number".into(),
                    ));
                }
                Ok(v.sqrt())
            }
            "fact" => {
                let v = expect_one(args)?;
                if v < 0.0 || v.fract() != 0.0 {
                    return Err(CalcError::DomainError(
                        "factorial requires a non-negative integer".into(),
                    ));
                }
                let n = v as u64;
                if n > 170 {
                    return Err(CalcError::Overflow);
                }
                let mut result: f64 = 1.0;
                for i in 2..=n {
                    result *= i as f64;
                }
                Ok(result)
            }

            // Other
            "abs" => {
                let v = expect_one(args)?;
                Ok(v.abs())
            }
            "floor" => {
                let v = expect_one(args)?;
                Ok(v.floor())
            }
            "ceil" => {
                let v = expect_one(args)?;
                Ok(v.ceil())
            }

            _ => Err(CalcError::InvalidExpression(format!(
                "Unknown function: {name}"
            ))),
        }
    }

    fn format_display(value: f64) -> String {
        if value.fract() == 0.0 && value.abs() < 1e15 {
            format!("{}", value as i64)
        } else {
            let s = format!("{:.10}", value);
            let s = s.trim_end_matches('0');
            let s = s.trim_end_matches('.');
            s.to_string()
        }
    }

    fn compute_alt_bases(value: f64) -> Option<AltBases> {
        if value.fract() == 0.0 && value >= 0.0 && value < u64::MAX as f64 {
            let n = value as u64;
            Some(AltBases {
                hex: format!("{:X}", n),
                oct: format!("{:o}", n),
                bin: format!("{:b}", n),
            })
        } else {
            None
        }
    }
}

impl Evaluate for EngineeringEngine {
    fn evaluate(&self, expr: &str) -> Result<CalcResult, CalcError> {
        let mut parser = Parser::new();
        let value = parser.parse_with_functions(expr, |name, args| {
            self.eval_function(name, args)
        })?;

        if value.is_infinite() {
            return Err(CalcError::Overflow);
        }
        if value.is_nan() {
            return Err(CalcError::DomainError("result is not a number".into()));
        }

        Ok(CalcResult {
            display: Self::format_display(value),
            alt_bases: Self::compute_alt_bases(value),
            value,
        })
    }
}
