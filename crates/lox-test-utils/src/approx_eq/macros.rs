#[macro_export]
macro_rules! approx_eq {
    ($lhs:expr, $rhs:expr) => {
        approx_eq!(
            $lhs,
            $rhs,
            atol <= 0.0,
            rtol <= $crate::approx_eq::default_rtol(0.0)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr) => {
        approx_eq!($lhs, $rhs, atol <= 0.0, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        approx_eq!(
            $lhs,
            $rhs,
            atol <= $atol,
            rtol <= $crate::approx_eq::default_rtol($atol)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr, atol <= $atol:expr) => {
        approx_eq!($lhs, $rhs, atol <= $atol, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr, rtol <= $rtol:expr) => {
        $crate::approx_eq::approx_eq_helper(&$lhs, &$rhs, $atol, $rtol).is_approx_eq()
    };
}

#[macro_export]
macro_rules! approx_ne {
    ($lhs:expr, $rhs:expr) => {
        approx_ne!(
            $lhs,
            $rhs,
            atol <= 0.0,
            rtol <= $crate::approx_eq::default_rtol(0.0)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr) => {
        approx_ne!($lhs, $rhs, atol <= 0.0, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        approx_ne!(
            $lhs,
            $rhs,
            atol <= $atol,
            rtol <= $crate::approx_eq::default_rtol($atol)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr, atol <= $atol:expr) => {
        approx_ne!($lhs, $rhs, atol <= $atol, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr, rtol <= $rtol:expr) => {
        $crate::approx_eq::approx_eq_helper(&$lhs, &$rhs, $atol, $rtol).is_approx_ne()
    };
}

#[macro_export]
macro_rules! assert_approx_eq {
    ($lhs:expr, $rhs:expr) => {
        assert_approx_eq!(
            $lhs,
            $rhs,
            atol <= 0.0,
            rtol <= $crate::approx_eq::default_rtol(0.0)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr) => {
        assert_approx_eq!($lhs, $rhs, atol <= 0.0, rtol <= $rtol);
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        assert_approx_eq!(
            &$lhs,
            &$rhs,
            atol <= $atol,
            rtol <= $crate::approx_eq::default_rtol($atol)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr, atol <= $atol:expr) => {
        assert_approx_eq!($lhs, $rhs, atol <= $atol, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr, rtol <= $rtol:expr) => {{
        let result = $crate::approx_eq::approx_eq_helper(&$lhs, &$rhs, $atol, $rtol);
        assert!(
            result.is_approx_eq(),
            "{:?} ≉ {:?}\n\nAbsolute tolerance: {:?}\nRelative tolerance: {:?}\n\n{}",
            $lhs,
            $rhs,
            $atol,
            $rtol,
            result
        )
    }};
}

#[macro_export]
macro_rules! assert_approx_ne {
    ($lhs:expr, $rhs:expr) => {
        assert_approx_ne!(
            &$lhs,
            &$rhs,
            atol <= 0.0,
            rtol <= $crate::approx_eq::default_rtol(0.0)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr) => {
        assert_approx_ne!($lhs, $rhs, 0.0, $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr) => {
        assert_approx_ne!(
            $lhs,
            $rhs,
            atol <= $atol,
            rtol <= $crate::approx_eq::default_rtol($atol)
        )
    };
    ($lhs:expr, $rhs:expr, rtol <= $rtol:expr, atol <= $atol:expr) => {
        assert_approx_ne!($lhs, $rhs, atol <= $atol, rtol <= $rtol)
    };
    ($lhs:expr, $rhs:expr, atol <= $atol:expr, rtol <= $rtol:expr) => {{
        let result = $crate::approx_eq::approx_eq_helper(&$lhs, &$rhs, $atol, $rtol);
        assert!(
            result.is_approx_ne(),
            "{:?} ≈ {:?}\n\nAbsolute tolerance: {:?}\nRelative tolerance: {:?}",
            $lhs,
            $rhs,
            $atol,
            $rtol,
        )
    }};
}
