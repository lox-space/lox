// SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
// SPDX-License-Identifier: MPL-2.0

use divan::Bencher;
use glam::{DMat3, DVec3};
use lox_test_utils::approx_eq;

fn main() {
    divan::main();
}

// ============================================================================
// F64 Benchmarks
// ============================================================================

#[divan::bench]
fn f64_default_tolerances_equal(bencher: Bencher) {
    let a = 1.0;
    let b = 1.0 + f64::EPSILON;
    bencher.bench(|| approx_eq!(a, b));
}

#[divan::bench]
fn f64_default_tolerances_not_equal(bencher: Bencher) {
    let a = 1.0;
    let b = 1.01;
    bencher.bench(|| approx_eq!(a, b));
}

#[divan::bench]
fn f64_custom_rtol_equal(bencher: Bencher) {
    let a = 1.0;
    let b = 1.001;
    bencher.bench(|| approx_eq!(a, b, rtol <= 0.01));
}

#[divan::bench]
fn f64_custom_rtol_not_equal(bencher: Bencher) {
    let a = 1.0;
    let b = 1.1;
    bencher.bench(|| approx_eq!(a, b, rtol <= 0.01));
}

#[divan::bench]
fn f64_custom_atol_equal(bencher: Bencher) {
    let a = 1.0;
    let b = 1.0001;
    bencher.bench(|| approx_eq!(a, b, atol <= 0.001));
}

#[divan::bench]
fn f64_custom_atol_not_equal(bencher: Bencher) {
    let a = 1.0;
    let b = 1.1;
    bencher.bench(|| approx_eq!(a, b, atol <= 0.001));
}

#[divan::bench]
fn f64_custom_both_tolerances_equal(bencher: Bencher) {
    let a = 1.0;
    let b = 1.0001;
    bencher.bench(|| approx_eq!(a, b, atol <= 0.001, rtol <= 0.01));
}

#[divan::bench]
fn f64_custom_both_tolerances_not_equal(bencher: Bencher) {
    let a = 1.0;
    let b = 1.1;
    bencher.bench(|| approx_eq!(a, b, atol <= 0.001, rtol <= 0.01));
}

// ============================================================================
// DVec3 Benchmarks
// ============================================================================

#[divan::bench]
fn dvec3_default_tolerances_equal(bencher: Bencher) {
    let v1 = DVec3::new(1.0, 2.0, 3.0);
    let v2 = DVec3::new(1.0 + f64::EPSILON, 2.0, 3.0);
    bencher.bench(|| approx_eq!(v1, v2));
}

#[divan::bench]
fn dvec3_default_tolerances_not_equal(bencher: Bencher) {
    let v1 = DVec3::new(1.0, 2.0, 3.0);
    let v2 = DVec3::new(1.01, 2.0, 3.0);
    bencher.bench(|| approx_eq!(v1, v2));
}

#[divan::bench]
fn dvec3_custom_tolerances_equal(bencher: Bencher) {
    let v1 = DVec3::new(1.0, 2.0, 3.0);
    let v2 = DVec3::new(1.001, 2.001, 3.001);
    bencher.bench(|| approx_eq!(v1, v2, atol <= 0.01, rtol <= 0.01));
}

#[divan::bench]
fn dvec3_custom_tolerances_not_equal(bencher: Bencher) {
    let v1 = DVec3::new(1.0, 2.0, 3.0);
    let v2 = DVec3::new(1.1, 2.1, 3.1);
    bencher.bench(|| approx_eq!(v1, v2, atol <= 0.01, rtol <= 0.01));
}

// ============================================================================
// DMat3 Benchmarks
// ============================================================================

#[divan::bench]
fn dmat3_default_tolerances_equal(bencher: Bencher) {
    let m1 = DMat3::from_cols(
        DVec3::new(1.0, 0.0, 0.0),
        DVec3::new(0.0, 1.0, 0.0),
        DVec3::new(0.0, 0.0, 1.0),
    );
    let m2 = DMat3::from_cols(
        DVec3::new(1.0 + f64::EPSILON, 0.0, 0.0),
        DVec3::new(0.0, 1.0, 0.0),
        DVec3::new(0.0, 0.0, 1.0),
    );
    bencher.bench(|| approx_eq!(m1, m2));
}

#[divan::bench]
fn dmat3_default_tolerances_not_equal(bencher: Bencher) {
    let m1 = DMat3::from_cols(
        DVec3::new(1.0, 0.0, 0.0),
        DVec3::new(0.0, 1.0, 0.0),
        DVec3::new(0.0, 0.0, 1.0),
    );
    let m2 = DMat3::from_cols(
        DVec3::new(1.01, 0.0, 0.0),
        DVec3::new(0.0, 1.0, 0.0),
        DVec3::new(0.0, 0.0, 1.0),
    );
    bencher.bench(|| approx_eq!(m1, m2));
}

#[divan::bench]
fn dmat3_custom_tolerances_equal(bencher: Bencher) {
    let m1 = DMat3::from_cols(
        DVec3::new(1.0, 0.0, 0.0),
        DVec3::new(0.0, 1.0, 0.0),
        DVec3::new(0.0, 0.0, 1.0),
    );
    let m2 = DMat3::from_cols(
        DVec3::new(1.001, 0.0, 0.0),
        DVec3::new(0.0, 1.001, 0.0),
        DVec3::new(0.0, 0.0, 1.001),
    );
    bencher.bench(|| approx_eq!(m1, m2, atol <= 0.01, rtol <= 0.01));
}

// ============================================================================
// Vec<f64> Benchmarks (Different Sizes)
// ============================================================================

#[divan::bench]
fn vec_small_3_elements_equal(bencher: Bencher) {
    let v1 = vec![1.0, 2.0, 3.0];
    let v2 = vec![1.0 + f64::EPSILON, 2.0, 3.0];
    bencher.bench(|| approx_eq!(v1, v2));
}

#[divan::bench]
fn vec_small_3_elements_not_equal(bencher: Bencher) {
    let v1 = vec![1.0, 2.0, 3.0];
    let v2 = vec![1.01, 2.0, 3.0];
    bencher.bench(|| approx_eq!(v1, v2));
}

#[divan::bench]
fn vec_medium_10_elements_equal(bencher: Bencher) {
    let v1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let v2 = vec![
        1.0 + f64::EPSILON,
        2.0,
        3.0,
        4.0,
        5.0,
        6.0,
        7.0,
        8.0,
        9.0,
        10.0,
    ];
    bencher.bench(|| approx_eq!(v1, v2));
}

#[divan::bench]
fn vec_medium_10_elements_not_equal(bencher: Bencher) {
    let v1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let v2 = vec![1.01, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    bencher.bench(|| approx_eq!(v1, v2));
}

#[divan::bench]
fn vec_large_100_elements_equal(bencher: Bencher) {
    let v1: Vec<f64> = (0..100).map(|i| i as f64).collect();
    let v2: Vec<f64> = (0..100).map(|i| i as f64 + f64::EPSILON).collect();
    bencher.bench(|| approx_eq!(v1, v2));
}

#[divan::bench]
fn vec_large_100_elements_not_equal(bencher: Bencher) {
    let v1: Vec<f64> = (0..100).map(|i| i as f64).collect();
    let v2: Vec<f64> = (0..100).map(|i| i as f64 + 0.01).collect();
    bencher.bench(|| approx_eq!(v1, v2));
}

// ============================================================================
// Array Benchmarks
// ============================================================================

#[divan::bench]
fn array_3_elements_equal(bencher: Bencher) {
    let a1 = [1.0, 2.0, 3.0];
    let a2 = [1.0 + f64::EPSILON, 2.0, 3.0];
    bencher.bench(|| approx_eq!(a1, a2));
}

#[divan::bench]
fn array_3_elements_not_equal(bencher: Bencher) {
    let a1 = [1.0, 2.0, 3.0];
    let a2 = [1.01, 2.0, 3.0];
    bencher.bench(|| approx_eq!(a1, a2));
}

#[divan::bench]
fn array_10_elements_equal(bencher: Bencher) {
    let a1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let a2 = [
        1.0 + f64::EPSILON,
        2.0,
        3.0,
        4.0,
        5.0,
        6.0,
        7.0,
        8.0,
        9.0,
        10.0,
    ];
    bencher.bench(|| approx_eq!(a1, a2));
}

#[divan::bench]
fn array_10_elements_not_equal(bencher: Bencher) {
    let a1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let a2 = [1.01, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    bencher.bench(|| approx_eq!(a1, a2));
}

// ============================================================================
// Worst-case Scenarios
// ============================================================================

#[divan::bench]
fn vec_early_difference_first_element(bencher: Bencher) {
    let v1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let v2 = vec![1.1, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    bencher.bench(|| approx_eq!(v1, v2));
}

#[divan::bench]
fn vec_late_difference_last_element(bencher: Bencher) {
    let v1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let v2 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.1];
    bencher.bench(|| approx_eq!(v1, v2));
}

#[divan::bench]
fn vec_all_elements_different(bencher: Bencher) {
    let v1: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let v2: Vec<f64> = (0..10).map(|i| i as f64 + 0.1).collect();
    bencher.bench(|| approx_eq!(v1, v2));
}
