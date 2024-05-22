use std::f64::consts::TAU;

use approx::abs_diff_eq;
use roots::{find_roots_cubic, find_roots_linear, find_roots_quadratic, find_roots_quartic, Roots};

fn roots_to_vec(roots: Roots<f64>) -> Vec<f64> {
    match roots {
        Roots::No(x) => x.to_vec(),
        Roots::One(x) => x.to_vec(),
        Roots::Two(x) => x.to_vec(),
        Roots::Three(x) => x.to_vec(),
        Roots::Four(x) => x.to_vec(),
    }
}

#[must_use]
#[inline]
pub fn solve_linear(a: impl Into<f64>, b: impl Into<f64>) -> Vec<f64> {
    let roots = find_roots_linear(a.into(), b.into());
    roots_to_vec(roots)
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_quadratic(a: impl Into<f64>, b: impl Into<f64>, c: impl Into<f64>) -> Vec<f64> {
    let roots = find_roots_quadratic(a.into(), b.into(), c.into());
    roots_to_vec(roots)
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_cubic(
    a: impl Into<f64>,
    b: impl Into<f64>,
    c: impl Into<f64>,
    d: impl Into<f64>,
) -> Vec<f64> {
    let roots = find_roots_cubic(a.into(), b.into(), c.into(), d.into());
    roots_to_vec(roots)
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_quartic(
    a: impl Into<f64>,
    b: impl Into<f64>,
    c: impl Into<f64>,
    d: impl Into<f64>,
    e: impl Into<f64>,
) -> Vec<f64> {
    let roots = find_roots_quartic(a.into(), b.into(), c.into(), d.into(), e.into());
    roots_to_vec(roots)
}

fn newton_method_with_derivative<F: Fn(f64) -> f64, D: Fn(f64) -> f64>(
    f: F,
    df: D,
    x0: f64,
    iterations: usize,
) -> f64 {
    let mut x = x0;
    for _ in 0..iterations {
        x -= f(x) / df(x);
    }
    x
}
fn newton_method<F: Fn(f64) -> f64>(f: F, x0: f64, iterations: usize) -> f64 {
    const DX: f64 = 0.1;
    let mut x = x0;
    for _ in 0..iterations {
        let df = f(x + DX) - f(x - DX) / (2.0 * DX);
        x -= f(x) / df;
    }
    x
}

#[cfg(test)]
mod tests {

    use approx::assert_abs_diff_eq;
    use roots::{find_roots_linear, Roots};

    use super::{
        newton_method_with_derivative, solve_cubic, solve_linear, solve_quadratic, solve_quartic,
    };
    use crate::root_finding::newton_method;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn test_linear(
            a in -1.0..1.0_f64,
            b in -1.0..1.0_f64,
        ) {
            _test_linear(a,b);
        }
        #[test]
        fn test_quadratic(
            a in -5.0..5.0_f64,
            b in -5.0..5.0_f64,
            c in -5.0..5.0_f64,
        ) {
            _test_quadratic(a,b,c);
        }
        #[test]
        fn test_cubic(
            a in -5.0..5.0_f64,
            b in -5.0..5.0_f64,
            c in -5.0..5.0_f64,
            d in -5.0..5.0_f64,
        ) {
            _test_cubic(a,b,c,d);
        }
        #[test]
        fn test_quartic(
            a in -5.0..5.0_f64,
            b in -5.0..5.0_f64,
            c in -5.0..5.0_f64,
            d in -5.0..5.0_f64,
            e in -5.0..5.0_f64,
        ) {
            _test_quartic(a,b,c,d,e);
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn _test_linear(a: f64, b: f64) {
        let solutions = solve_linear(a, b);

        let f = |x: f64| a.mul_add(x, b);
        println!("a={a} b={b} solutions = {solutions:?}");

        for x in solutions {
            assert_abs_diff_eq!(f(x), 0.0, epsilon = 1e-5);
        }
    }
    #[allow(clippy::many_single_char_names)]
    fn _test_quadratic(a: f64, b: f64, c: f64) {
        let solutions = solve_quadratic(a, b, c);

        let f = |x: f64| {
            let x2 = x * x;
            a.mul_add(x2, b * x) + c
        };
        println!("a={a} b={b},c={c} solutions = {solutions:?}");
        for x in solutions {
            assert_abs_diff_eq!(f(x), 0.0, epsilon = 1e-5);
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn _test_cubic(a: f64, b: f64, c: f64, d: f64) {
        let solutions = solve_cubic(a, b, c, d);
        let f = |x: f64| {
            let x2 = x * x;
            let x3 = x2 * x;
            c.mul_add(x, a.mul_add(x3, b * x2)) + d
        };
        for x in solutions {
            assert_abs_diff_eq!(f(x), 0.0, epsilon = 1e-5);
        }
    }
    #[allow(clippy::many_single_char_names)]
    fn _test_quartic(a: f64, b: f64, c: f64, d: f64, e: f64) {
        let solutions = solve_quartic(a, b, c, d, e);
        let f = |x: f64| {
            let x2 = x * x;
            let x3 = x2 * x;
            let x4 = x3 * x;
            d.mul_add(x, c.mul_add(x2, a.mul_add(x4, b * x3))) + e
        };
        for x in solutions {
            assert_abs_diff_eq!(f(x), 0.0, epsilon = 1e-5);
        }
    }
    #[test]
    fn test_newton_method() {
        let f = |x: f64| x.ln() + x - 7.0;
        let df = |x: f64| x.recip() + 1.0;
        let root = newton_method_with_derivative(f, df, 5.0, 20);
        assert_abs_diff_eq!(f(root), 0.0, epsilon = 1e-4);
    }
    #[test]
    fn test_newton_method_with_derivative() {
        let f = |x: f64| x.ln() + x - 7.0;
        let root = newton_method(f, 5.0, 20);
        assert_abs_diff_eq!(f(root), 0.0, epsilon = 1e-4);
    }
}
