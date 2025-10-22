// SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

use libm::tgamma;

pub fn c2(psi: f64) -> f64 {
    let eps = 1.0;
    if psi > eps {
        (1.0 - psi.sqrt().cos()) / psi
    } else if psi < -eps {
        ((-psi).sqrt().cosh() - 1.0) / (-psi)
    } else {
        let mut res = 1.0 / 2.0;
        let mut delta = (-psi) / tgamma(2.0 + 2.0 + 1.0);
        let mut k = 1;
        while res + delta != res {
            res += delta;
            k += 1;
            delta = (-psi).powi(k) / tgamma(2.0 * k as f64 + 2.0 + 1.0)
        }
        res
    }
}

pub fn c3(psi: f64) -> f64 {
    let eps = 1.0;
    if psi > eps {
        (psi.sqrt() - psi.sqrt().sin()) / (psi * psi.sqrt())
    } else if psi < -eps {
        ((-psi).sqrt().sinh() - ((-psi).sqrt())) / (-psi * (-psi).sqrt())
    } else {
        let mut res = 1.0 / 6.0;
        let mut delta = -psi / tgamma(2.0 + 3.0 + 1.0);
        let mut k = 1;
        while res + delta != res {
            res += delta;
            k += 1;
            delta = (-psi).powi(k) / tgamma(2.0 * k as f64 + 3.0 + 1.0)
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use lox_test_utils::assert_approx_eq;

    use super::*;

    #[test]
    fn stumpff_near_zero() {
        let psi = 0.5f64;
        let expected_c2 = (1.0 - psi.powf(0.5).cos()) / psi;
        let expected_c3 = (psi.powf(0.5) - psi.powf(0.5).sin()) / psi.powf(1.5);

        assert_approx_eq!(c2(psi), expected_c2, rtol <= 1e-8);
        assert_approx_eq!(c3(psi), expected_c3, rtol <= 1e-8);
    }

    #[test]
    fn test_stumpff_functions_above_zero() {
        let psi = 3.0f64;
        let expected_c2 = (1.0 - psi.powf(0.5).cos()) / psi;
        let expected_c3 = (psi.powf(0.5) - psi.powf(0.5).sin()) / psi.powf(1.5);

        assert_approx_eq!(c2(psi), expected_c2, rtol <= 1e-10);
        assert_approx_eq!(c3(psi), expected_c3, rtol <= 1e-10);
    }

    #[test]
    fn test_stumpff_functions_under_zero() {
        let psi = -3.0f64;
        let expected_c2 = ((-psi).powf(0.5).cosh() - 1.0) / (-psi);
        let expected_c3 = ((-psi).powf(0.5).sinh() - (-psi).powf(0.5)) / (-psi).powf(1.5);

        assert_approx_eq!(c2(psi), expected_c2, rtol <= 1e-10);
        assert_approx_eq!(c3(psi), expected_c3, rtol <= 1e-10);
    }
}
