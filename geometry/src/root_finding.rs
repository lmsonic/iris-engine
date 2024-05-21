use std::f32::consts::TAU;

use num_complex::Complex;

#[derive(Clone, Copy, Debug)]
pub enum QuadraticSolution {
    TwoReal(f32, f32),
    OneReal(f32),
    TwoComplex(Complex<f32>, Complex<f32>),
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_quadratic(a: f32, b: f32, c: f32) -> QuadraticSolution {
    let discr = b.mul_add(b, -(4.0 * a * c));
    if discr >= 0.0 {
        if discr == 0.0 {
            let x = (-b) / (2.0 * a);
            QuadraticSolution::OneReal(x)
        } else {
            let d = discr.sqrt();
            let x1 = (-b - d) / (2.0 * a); // smallest
            let x2 = (-b + d) / (2.0 * a); // largest
            QuadraticSolution::TwoReal(x1, x2)
        }
    } else {
        let imag = discr.abs().sqrt() / (2.0 * a);
        let real = -b / (2.0 * a);
        QuadraticSolution::TwoComplex(Complex::new(real, imag), Complex::new(real, -imag))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CubicSolution {
    ThreeReal(f32, f32, f32),
    TwoReal(f32, f32),
    OneReal(f32, Complex<f32>, Complex<f32>),
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_cubic(a_: f32, b: f32, c: f32, d: f32) -> CubicSolution {
    // Normalization
    let a = b / a_;
    let b = c / a_;
    let c = d / a_;
    // Remove quadratic term by substituting x'=x-a/3
    let substitution_term = -a / 3.0;
    let p = -a * a / 3.0 + b;
    let q = 2.0 * a.powi(3) / 27.0 - a * b / 3.0 + c;

    let p = p / 3.0;
    let q = q / 2.0;
    let rho = Complex::new(-0.5, f32::sqrt(3.0) * 0.5);
    let rho2 = rho * rho;
    let discrim = -p.powi(3) - q * q;
    if discrim < 0.0 {
        let d = (-discrim).sqrt();
        let r = (-q + d).cbrt();
        let s = (-q - d).cbrt();

        let x1 = r + s + substitution_term;
        let x2 = rho * r + rho2 * s + substitution_term;
        let x3 = rho2 * r + rho * s + substitution_term;
        CubicSolution::OneReal(x1, x2, x3)
    } else if discrim == 0.0 {
        let r = (-q).cbrt();
        let x1 = 2.0 * r + substitution_term;
        let x2 = -r + substitution_term;
        CubicSolution::TwoReal(x1, x2)
    } else {
        let theta = f32::acos(-q / (-p).powi(3).sqrt());
        let two_m = 2.0 * (-p).sqrt();
        let x1 = two_m * theta.cos() + substitution_term;
        let x2 = two_m * (theta + TAU / 3.0).cos() + substitution_term;
        let x3 = two_m * (theta - TAU / 3.0).cos() + substitution_term;

        CubicSolution::ThreeReal(x1, x2, x3)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum QuarticSolution {
    FourReal(f32, f32, f32, f32),
    ThreeReal(f32, f32, f32),
    TwoRealTwoComplex(f32, f32, Complex<f32>, Complex<f32>),
    TwoReal(f32, f32),
    OneRealTwoComplex(f32, Complex<f32>, Complex<f32>),
    FourComplex(Complex<f32>, Complex<f32>, Complex<f32>, Complex<f32>),
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn solve_quartic(a: f32, b: f32, c: f32, d: f32, e: f32) -> QuarticSolution {
    // Normalization
    let b = c / a;
    let c = d / a;
    let d = e / a;
    let a = b / a;
    // Remove cubic term by substituting x'=x-a/4
    let substitution_term = -a / 4.0;
    let p = -a * a * 3.0 / 8.0 + b;
    let q = a.powi(3) / 8.0 - a * b * 0.5 + c;
    let r = -3.0 * a.powi(4) / 256.0 + a * a * b / 16.0 - a * c / 4.0 + d;

    let cubic_solution = solve_cubic(1.0, -p / 2.0, -r, (-4.0 * r * p - q * q) / 8.0);
    let y = match cubic_solution {
        CubicSolution::TwoReal(x, _)
        | CubicSolution::ThreeReal(x, _, _)
        | CubicSolution::OneReal(x, _, _) => x,
    };
    let (solution1, solution2) = if q >= 0.0 {
        let solution1 = solve_quadratic(1.0, (2.0 * y - p).sqrt(), y - (y * y - r).sqrt());
        let solution2 = solve_quadratic(1.0, -(2.0 * y - p).sqrt(), y + (y * y - r).sqrt());
        (solution1, solution2)
    } else {
        let solution1 = solve_quadratic(1.0, (2.0 * y - p).sqrt(), y + (y * y - r).sqrt());
        let solution2 = solve_quadratic(1.0, -(2.0 * y - p).sqrt(), y - (y * y - r).sqrt());
        (solution1, solution2)
    };
    let solution1 = match solution1 {
        QuadraticSolution::TwoReal(x1, x2) => {
            QuadraticSolution::TwoReal(x1 + substitution_term, x2 + substitution_term)
        }
        QuadraticSolution::OneReal(x) => QuadraticSolution::OneReal(x + substitution_term),
        QuadraticSolution::TwoComplex(c1, c2) => {
            QuadraticSolution::TwoComplex(c1 + substitution_term, c2 + substitution_term)
        }
    };
    let solution2 = match solution2 {
        QuadraticSolution::TwoReal(x1, x2) => {
            QuadraticSolution::TwoReal(x1 + substitution_term, x2 + substitution_term)
        }
        QuadraticSolution::OneReal(x) => QuadraticSolution::OneReal(x + substitution_term),
        QuadraticSolution::TwoComplex(c1, c2) => {
            QuadraticSolution::TwoComplex(c1 + substitution_term, c2 + substitution_term)
        }
    };
    match (solution1, solution2) {
        (QuadraticSolution::TwoReal(x1, x2), QuadraticSolution::TwoReal(x3, x4)) => {
            QuarticSolution::FourReal(x1, x2, x3, x4)
        }
        (QuadraticSolution::TwoReal(x1, x2), QuadraticSolution::OneReal(x3))
        | (QuadraticSolution::OneReal(x1), QuadraticSolution::TwoReal(x2, x3)) => {
            QuarticSolution::ThreeReal(x1, x2, x3)
        }
        (QuadraticSolution::TwoReal(x1, x2), QuadraticSolution::TwoComplex(c1, c2))
        | (QuadraticSolution::TwoComplex(c1, c2), QuadraticSolution::TwoReal(x1, x2)) => {
            QuarticSolution::TwoRealTwoComplex(x1, x2, c1, c2)
        }

        (QuadraticSolution::OneReal(x1), QuadraticSolution::OneReal(x2)) => {
            QuarticSolution::TwoReal(x1, x2)
        }
        (QuadraticSolution::OneReal(x), QuadraticSolution::TwoComplex(c1, c2))
        | (QuadraticSolution::TwoComplex(c1, c2), QuadraticSolution::OneReal(x)) => {
            QuarticSolution::OneRealTwoComplex(x, c1, c2)
        }

        (QuadraticSolution::TwoComplex(c1, c2), QuadraticSolution::TwoComplex(c3, c4)) => {
            QuarticSolution::FourComplex(c1, c2, c3, c4)
        }
    }
}

fn newton_method_with_derivative<F: Fn(f32) -> f32, D: Fn(f32) -> f32>(
    f: F,
    df: D,
    x0: f32,
    iterations: usize,
) -> f32 {
    let mut x = x0;
    for _ in 0..iterations {
        x -= f(x) / df(x);
    }
    x
}
fn newton_method<F: Fn(f32) -> f32>(f: F, x0: f32, iterations: usize) -> f32 {
    const DX: f32 = 0.1;
    let mut x = x0;
    for _ in 0..iterations {
        let df = f(x + DX) - f(x - DX) / (2.0 * DX);
        x -= f(x) / df;
    }
    x
}

#[cfg(test)]
mod tests {

    use approx::{assert_abs_diff_eq, assert_relative_eq};

    use super::{
        newton_method_with_derivative, solve_cubic, solve_quadratic, CubicSolution,
        QuadraticSolution,
    };
    use crate::root_finding::newton_method;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn test_quadratic(
            a in -1000.0..1000.0_f32,
            b in -1000.0..1000.0_f32,
            c in -1000.0..1000.0_f32,
        ) {
            prop_assume!(a != 0.0);
            _test_quadratic(a,b,c);
        }
        #[test]
        fn test_cubic(
            a in -1000.0..1000.0_f32,
            b in -1000.0..1000.0_f32,
            c in -1000.0..1000.0_f32,
            d in -1000.0..1000.0_f32,
        ) {
            prop_assume!(a != 0.0);
            _test_cubic(a,b,c,d);
        }
    }
    #[allow(clippy::many_single_char_names)]
    fn _test_quadratic(a: f32, b: f32, c: f32) {
        let solution = solve_quadratic(a, b, c);
        let f = |x: f32| (a * x).mul_add(x, b * x) + c;
        match solution {
            QuadraticSolution::TwoReal(x1, x2) => {
                assert_abs_diff_eq!(f(x1), 0.0, epsilon = 0.01);
                assert_abs_diff_eq!(f(x2), 0.0, epsilon = 0.01);
            }
            QuadraticSolution::OneReal(x) => {
                assert_abs_diff_eq!(f(x), 0.0, epsilon = 0.01);
            }
            QuadraticSolution::TwoComplex(_, _) => {}
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn _test_cubic(a: f32, b: f32, c: f32, d: f32) {
        let solution = solve_cubic(a, b, c, d);
        let f = |x: f32| (a * x.powi(3) * b * x).mul_add(x, c * x) + d;
        match solution {
            CubicSolution::ThreeReal(x1, x2, x3) => {
                assert_relative_eq!(f(x1), 0.0, epsilon = 0.01);
                assert_abs_diff_eq!(f(x2), 0.0, epsilon = 0.01);
                assert_abs_diff_eq!(f(x3), 0.0, epsilon = 0.01);
            }
            CubicSolution::TwoReal(x1, x2) => {
                assert_relative_eq!(f(x1), 0.0, epsilon = 0.01);
                assert_abs_diff_eq!(f(x2), 0.0, epsilon = 0.01);
            }
            CubicSolution::OneReal(x, _, _) => {
                assert_abs_diff_eq!(f(x), 0.0, epsilon = 0.01);
            }
        }
    }
    #[test]
    fn test_newton_method() {
        let f = |x: f32| x.ln() + x - 7.0;
        let df = |x: f32| x.recip() + 1.0;
        let root = newton_method_with_derivative(f, df, 5.0, 20);
        assert_abs_diff_eq!(f(root), 0.0, epsilon = 1e-4);
    }
    #[test]
    fn test_newton_method_with_derivative() {
        let f = |x: f32| x.ln() + x - 7.0;
        let root = newton_method(f, 5.0, 20);
        assert_abs_diff_eq!(f(root), 0.0, epsilon = 1e-4);
    }
}
