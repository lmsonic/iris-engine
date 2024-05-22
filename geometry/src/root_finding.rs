use std::f64::consts::TAU;

use approx::abs_diff_eq;

#[must_use]
#[inline]
pub fn solve_linear(a: impl Into<f64>, b: impl Into<f64>) -> Vec<f64> {
    let a = a.into();
    let b = b.into();
    if abs_diff_eq!(a, 0.0) {
        if abs_diff_eq!(b, 0.0) {
            vec![b]
        } else {
            vec![]
        }
    } else {
        vec![-b / a]
    }
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_quadratic(a: impl Into<f64>, b: impl Into<f64>, c: impl Into<f64>) -> Vec<f64> {
    let a = a.into();
    let b = b.into();
    let c = c.into();
    if abs_diff_eq!(a, 0.0) {
        solve_linear(b, c)
    } else {
        // https://people.csail.mit.edu/bkph/articles/Quadratics.pdf
        let b = b / a;
        let c = c / a;
        let discr = b * b - 4.0 * a * c;
        if abs_diff_eq!(discr, 0.0) {
            let x = -b * 0.5;
            vec![x]
        } else if discr >= 0.0 {
            let d = discr.sqrt();
            // We avoid substraction for umerical stability
            if b < 0.0 {
                let x1 = 2.0 * c / (-b + d);
                let x2 = (-b + d) * 0.5;
                vec![x1, x2]
            } else {
                let x1 = (-b - d) * 0.5;
                let x2 = 2.0 * c / (-b - d);
                vec![x1, x2]
            }
        } else {
            vec![]
        }
    }
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_cubic(
    a: impl Into<f64>,
    b: impl Into<f64>,
    c: impl Into<f64>,
    d: impl Into<f64>,
) -> Vec<f64> {
    let a = a.into();
    let b = b.into();
    let c = c.into();
    let d = d.into();
    if abs_diff_eq!(d, 0.0) {
        // First solution is x = 0
        // Divide all terms by x, converting to quadratic equation

        return [solve_quadratic(a, b, c), vec![0.0]].concat();
    }
    if abs_diff_eq!(a, 0.0) {
        return solve_quadratic(b, c, d);
    }

    // Normalization
    // new equation :  x^3+ a*x^2 + b*x + c;
    let a_ = a;
    let a = b / a_;
    let b = c / a_;
    let c = d / a_;
    // x' = x - b/3.0
    let subst = a / 3.0;
    let subst2 = subst * subst;
    let subst3 = subst2 * subst;

    let p = -subst2 + b / 3.0;
    let q = subst3 - b * subst * 0.5 + c * 0.5;

    let discr = -(p.powi(3) + q * q);
    let mut solutions = if abs_diff_eq!(discr, 0.1) {
        let r = (-q).cbrt();
        let x1 = 2.0 * r;
        let x2 = -r;
        println!("second");
        vec![x1, x2]
    } else if discr < 0.0 {
        let d = (-discr).sqrt();
        let r = (-q + d).cbrt();
        let s = (-q - d).cbrt();
        let x = r + s;
        println!("first");
        vec![x]
    } else {
        let theta = f64::acos(-q / (-p).powi(3).sqrt()) / 3.0;
        let two_m = 2.0 * (-p).sqrt();
        let x1 = two_m * theta.cos();
        let x2 = two_m * (theta + TAU / 3.0).cos();
        let x3 = two_m * (theta - TAU / 3.0).cos();
        println!("third");
        vec![x1, x2, x3]
    };
    for x in &mut solutions {
        *x -= subst;
    }
    solutions
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
    let a = a.into();
    let b = b.into();
    let c = c.into();
    let d = d.into();
    let e = e.into();
    if abs_diff_eq!(e, 0.0) {
        // First solution is x = 0
        // Divide all terms by x, converting to quadratic equation

        return [solve_cubic(a, b, c, d), vec![0.0]].concat();
    }
    if abs_diff_eq!(a, 0.0) {
        return solve_cubic(b, c, d, e);
    }
    // Normalization
    // new equation :  x^4+ a*x^4 + b*x^2 + c*x + d;
    let a_ = a;
    let a = b / a_;
    let b = c / a_;
    let c = d / a_;
    let d = e / a_;

    // Remove cubic term by substituting x'=x-a/4
    let subst = a * 0.25;
    let subst2 = subst * subst;
    let subst4 = subst2 * subst2;
    let p = -a * 3.0 * 0.5 * subst + b;
    let q = a * a * subst * 0.5 - a * b * 0.5 + c;
    let r = -3.0 * subst4 + subst2 * b - subst * c + d;

    let cubic_solutions = solve_cubic(1.0, -p * 0.5, -r, (4.0 * r * p - q * q) / 8.0);
    let Some(y) = cubic_solutions.first() else {
        return vec![];
    };

    let b = (2.0 * y - p).sqrt();
    let c = (y * y - r).sqrt();
    let mut solutions = if q >= 0.0 {
        [
            solve_quadratic(1.0, b, y - c),
            solve_quadratic(1.0, -b, y + c),
        ]
        .concat()
    } else {
        [
            solve_quadratic(1.0, b, y + c),
            solve_quadratic(1.0, -b, y - c),
        ]
        .concat()
    };
    for x in &mut solutions {
        *x -= subst;
    }
    solutions
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
        // #[test]
        // fn test_cubic(
        //     a in -5.0..5.0_f64,
        //     b in -5.0..5.0_f64,
        //     c in -5.0..5.0_f64,
        //     d in -5.0..5.0_f64,
        // ) {
        //     _test_cubic(a,b,c,d);
        // }
        // #[test]
        // fn test_quartic(
        //     a in -5.0..5.0_f64,
        //     b in -5.0..5.0_f64,
        //     c in -5.0..5.0_f64,
        //     d in -5.0..5.0_f64,
        //     e in -5.0..5.0_f64,
        // ) {
        //     _test_quartic(a,b,c,d,e);
        // }
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
    // #[allow(clippy::many_single_char_names)]
    // #[test]
    // fn test_cubic_1() {
    //     let a = -4.9253645;
    //     let b = 0.0;
    //     let c = -0.011813866;
    //     let d = -3.4115908;
    //     let solutions = solve_cubic(a, b, c, d);

    //     let f = |x: f64| {
    //         let x2 = x * x;
    //         a * x2 + b * x + c
    //     };
    //     for x in solutions {
    //         assert_abs_diff_eq!(f(x), 0.0);
    //     }
    // }

    #[allow(clippy::many_single_char_names)]
    fn _test_cubic(a: f64, b: f64, c: f64, d: f64) {
        let solutions = solve_cubic(a, b, c, d);
        let f = |x: f64| {
            let x2 = x * x;
            let x3 = x2 * x;
            c.mul_add(x, a.mul_add(x3, b * x2)) + d
        };
        for x in solutions {
            assert_abs_diff_eq!(f(x), 0.0);
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
            assert_abs_diff_eq!(f(x), 0.0);
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
