use std::ops::RangeInclusive;

use roots::{
    find_root_brent, find_root_newton_raphson, find_roots_cubic, find_roots_linear,
    find_roots_quadratic, find_roots_quartic, FloatType, Roots, SimpleConvergency,
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

#[inline]
pub fn solve_linear<F: FloatType>(a: F, b: F) -> Vec<F> {
    let roots = find_roots_linear(a, b);
    roots_to_vec(&roots)
}

#[allow(clippy::many_single_char_names)]
pub fn solve_quadratic<F: FloatType>(a: F, b: F, c: F) -> Vec<F> {
    let roots = find_roots_quadratic(a, b, c);
    roots_to_vec(&roots)
}

#[allow(clippy::many_single_char_names)]
pub fn solve_cubic<F: FloatType>(a: F, b: F, c: F, d: F) -> Vec<F> {
    let roots = find_roots_cubic(a, b, c, d);
    roots_to_vec(&roots)
}

#[allow(clippy::many_single_char_names)]
pub fn solve_quartic<F: FloatType>(a: F, b: F, c: F, d: F, e: F) -> Vec<F> {
    let roots = find_roots_quartic(a, b, c, d, e);
    roots_to_vec(&roots)
}

pub fn newton_method<F: Fn(f64) -> f64, D: Fn(f64) -> f64>(
    f: F,
    df: D,
    x0: f64,
    iterations: usize,
) -> Option<f64> {
    find_root_newton_raphson(
        x0,
        f,
        df,
        &mut SimpleConvergency {
            eps: 1e-2,
            max_iter: iterations,
        },
    )
    .ok()
}
pub fn brent_method<F: FnMut(f64) -> f64>(
    f: F,
    range: RangeInclusive<f64>,
    iterations: usize,
) -> Option<f64> {
    let result = find_root_brent(
        *range.start(),
        *range.end(),
        f,
        &mut SimpleConvergency {
            eps: 1e-2,
            max_iter: iterations,
        },
    );
    result.ok()
}

#[cfg(test)]
mod tests {

    use std::ops::RangeInclusive;

    use approx::assert_abs_diff_eq;

    use super::{brent_method, solve_cubic, solve_linear, solve_quadratic, solve_quartic};
    use crate::geometry::root_finding::newton_method;
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
        #[test]
        fn test_newton_cubic(
            a in RANGE,
            b in RANGE,
            c in RANGE,
            d in RANGE,
        ) {
            _test_newton_cubic(a,b,c,d);
        }



    }
    fn _test_newton_quadratic(a: f64, b: f64, c: f64) {
        let solutions = solve_quadratic(a, b, c);
        let f = |x: f64| {
            let x2 = x * x;
            a.mul_add(x2, b * x) + c
        };
        let df = |x: f64| (2.0 * a).mul_add(x, b);
        test_newton_method(f, df, 40, &solutions, 0.2);
    }
    #[allow(clippy::many_single_char_names)]
    fn _test_newton_cubic(a: f64, b: f64, c: f64, d: f64) {
        let solutions = solve_cubic(a, b, c, d);
        let f = |x: f64| {
            let x2 = x * x;
            let x3 = x2 * x;
            c.mul_add(x, a.mul_add(x3, b * x2)) + d
        };
        let df = |x: f64| {
            let x2 = x * x;
            (3.0 * a).mul_add(x2, 2.0 * b * x) + c
        };
        test_newton_method(f, df, 40, &solutions, 0.5);
    }

    fn test_newton_method<F: Fn(f64) -> f64, D: Fn(f64) -> f64>(
        f: F,
        df: D,
        iterations: usize,
        solutions: &[f64],
        e: f64,
    ) {
        for x in solutions {
            let root = newton_method(&f, &df, *x + 0.1, iterations).unwrap();
            assert_abs_diff_eq!(f(root), 0.0, epsilon = e);
            assert_abs_diff_eq!(root, x, epsilon = e);
        }
    }

    #[test]
    fn test_newton_method_log() {
        let f = |x: f64| x.ln() + x - 7.0;
        let df = |x: f64| x.recip() + 1.0;
        let root = newton_method(f, df, 6.0, 20).unwrap();
        assert_abs_diff_eq!(f(root), 0.0, epsilon = 1e-1);
    }
    #[test]
    fn test_bisection_method_log() {
        let f = |x: f64| x.ln() + x - 7.0;
        let root = brent_method(f, 4.0..=6.0, 20).unwrap();
        assert_abs_diff_eq!(f(root), 0.0, epsilon = 1e-1);
    }
}
