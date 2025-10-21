use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
pub enum ApproxEqResult {
    Pass,
    Fail {
        left: f64,
        right: f64,
        diff: Option<f64>,
        tol: Option<f64>,
    },
}

impl ApproxEqResult {
    pub fn new(left: f64, right: f64, atol: f64, rtol: f64) -> Self {
        if !left.is_finite() || !right.is_finite() {
            return Self::Fail {
                left,
                right,
                diff: None,
                tol: None,
            };
        }
        // Effective tolerance
        let tol = f64::max(atol, rtol * f64::max(left.abs(), right.abs()));
        let diff = (left - right).abs();
        if diff > tol {
            Self::Fail {
                left,
                right,
                diff: Some(diff),
                tol: Some(tol),
            }
        } else {
            Self::Pass
        }
    }
}

#[derive(Debug, Default)]
pub struct ApproxEqResults(HashMap<String, ApproxEqResult>);

impl ApproxEqResults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_approx_eq(&self) -> bool {
        self.0.values().all(|v| matches!(v, ApproxEqResult::Pass))
    }

    pub fn is_approx_ne(&self) -> bool {
        !self.is_approx_eq()
    }

    pub fn insert(&mut self, field: String, result: ApproxEqResult) -> &mut Self {
        self.0.insert(field, result);
        self
    }

    pub fn merge(&mut self, field: impl AsRef<str>, other: Self) -> &mut Self {
        let field = field.as_ref().to_owned();
        for (other_field, result) in other.0 {
            let field = if !other_field.is_empty() {
                format!("{}.{}", field.clone(), other_field)
            } else {
                field.clone()
            };
            self.0.insert(field, result);
        }
        self
    }
}

impl Display for ApproxEqResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (field, result) in &self.0 {
            match result {
                ApproxEqResult::Pass => continue,
                ApproxEqResult::Fail {
                    left,
                    right,
                    diff,
                    tol,
                } => {
                    if !field.is_empty() {
                        writeln!(f, "Field: {}", field)?;
                    }
                    writeln!(f, "Left:  {:?}", left)?;
                    writeln!(f, "Right: {:?}", right)?;
                    if let Some(diff) = diff {
                        writeln!(f, "Diff:  {:?}", diff)?;
                    }
                    if let Some(tol) = tol {
                        writeln!(f, "Tol:   {:?}\n", tol)?;
                    }
                }
            }
        }
        write!(f, "")
    }
}
