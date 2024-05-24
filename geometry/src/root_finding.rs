use approx::abs_diff_eq;
use roots::{
    find_roots_cubic, find_roots_linear, find_roots_quadratic, find_roots_quartic, FloatType, Roots,
};

fn roots_to_vec<F: FloatType>(roots: &Roots<F>) -> Vec<F> {
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
pub fn solve_linear<F: FloatType>(a: F, b: F) -> Vec<F> {
    let roots = find_roots_linear(a, b);
    roots_to_vec(&roots)
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_quadratic<F: FloatType>(a: F, b: F, c: F) -> Vec<F> {
    let roots = find_roots_quadratic(a, b, c);
    roots_to_vec(&roots)
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_cubic<F: FloatType>(a: F, b: F, c: F, d: F) -> Vec<F> {
    let roots = find_roots_cubic(a, b, c, d);
    roots_to_vec(&roots)
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_quartic<F: FloatType>(a: F, b: F, c: F, d: F, e: F) -> Vec<F> {
    let roots = find_roots_quartic(a, b, c, d, e);
    roots_to_vec(&roots)
}

fn newton_method_with_derivative<F: Fn(f64) -> f64, D: Fn(f64) -> f64>(
    f: F,
    df: D,
    x0: f64,
    iterations: usize,
) -> f64 {
    let mut x = x0;
    for _ in 0..iterations {
        println!("{x} {}", f(x));
        x -= f(x) / df(x);
        if abs_diff_eq!(f(x), 0.0, epsilon = 1e-2) {
            break;
        }
    }
    x
}
fn newton_method<F: Fn(f64) -> f64>(f: F, x0: f64, iterations: usize) -> f64 {
    const DX: f64 = 0.1;
    let mut x = x0;
    for _ in 0..iterations {
        let df = (f(x + DX) - f(x - DX)) / (2.0 * DX);
        x -= f(x) / df;
        if abs_diff_eq!(f(x), 0.0, epsilon = 1e-2) {
            break;
        }
    }
    x
}

#[cfg(test)]
mod tests {

    use std::ops::RangeInclusive;

    use approx::assert_abs_diff_eq;

    use super::{
        newton_method_with_derivative, solve_cubic, solve_linear, solve_quadratic, solve_quartic,
    };
    use crate::root_finding::newton_method;
    use proptest::prelude::*;
    const RANGE: RangeInclusive<f64> = -1.0..=1.0;

    proptest! {
        #[test]
        fn test_linear(
            a in RANGE,
            b in RANGE,
        ) {
            _test_linear(a,b);
        }
        #[test]
        fn test_quadratic(
            a in RANGE,
            b in RANGE,
            c in RANGE,
        ) {
            _test_quadratic(a,b,c);
        }
        #[test]
        fn test_cubic(
            a in RANGE,
            b in RANGE,
            c in RANGE,
            d in RANGE,
        ) {
            _test_cubic(a,b,c,d);
        }
        #[test]
        fn test_quartic(
            a in RANGE,
            b in RANGE,
            c in RANGE,
            d in RANGE,
            e in RANGE,
        ) {
            _test_quartic(a,b,c,d,e);
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn _test_linear(a: f64, b: f64) {
        let solutions = solve_linear(a, b);

        let f = |x: f64| a.mul_add(x, b);

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
            assert_abs_diff_eq!(f(x), 0.0, epsilon = 1e-2);
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
            assert_abs_diff_eq!(f(x), 0.0, epsilon = 1e-1);
        }
    }
    proptest! {
        #[test]
        fn test_newton_quadratic(
            a in RANGE,
            b in RANGE,
            c in RANGE,
        ) {
            _test_newton_quadratic(a,b,c);
        }
    }
    fn _test_newton_quadratic(a: f64, b: f64, c: f64) {
        let solution = solve_quadratic(a, b, c);
        let f = |x: f64| {
            let x2 = x * x;
            a.mul_add(x2, b * x) + c
        };
        let df = |x: f64| (2.0 * a).mul_add(x, b);
        if !solution.is_empty() {
            test_newton_method(f, 0.0, 20);
            test_newton_method_with_derivative(f, df, 0.0, 20);
        }
    }
    #[allow(clippy::many_single_char_names)]
    fn _test_newton_cubic(a: f64, b: f64, c: f64, d: f64) {
        let solution = solve_cubic(a, b, c, d);
        let f = |x: f64| {
            let x2 = x * x;
            let x3 = x2 * x;
            c.mul_add(x, a.mul_add(x3, b * x2)) + d
        };
        let df = |x: f64| {
            let x2 = x * x;
            (3.0 * a).mul_add(x2, 2.0 * b * x) + c
        };
        if !solution.is_empty() {
            test_newton_method(f, 0.0, 20);
            test_newton_method_with_derivative(f, df, 0.0, 20);
        }
    }

    fn test_newton_method_with_derivative<F: Fn(f64) -> f64, D: Fn(f64) -> f64>(
        f: F,
        df: D,
        x0: f64,
        iterations: usize,
    ) {
        let root = newton_method_with_derivative(&f, df, x0, iterations);
        assert_abs_diff_eq!(f(root), 0.0, epsilon = 1e-2);
    }
    fn test_newton_method<F: Fn(f64) -> f64>(f: F, x0: f64, iterations: usize) {
        let root = newton_method(&f, x0, iterations);
        assert_abs_diff_eq!(f(root), 0.0, epsilon = 1e-2);
    }
    #[test]
    fn test_newton_method_derivative_log() {
        let f = |x: f64| x.ln() + x - 7.0;
        let df = |x: f64| x.recip() + 1.0;
        test_newton_method_with_derivative(f, df, 6.0, 20);
    }
    #[test]
    fn test_newton_method_log() {
        let f = |x: f64| x.ln() + x - 7.0;
        test_newton_method(f, 6.0, 20);
    }
}
