pub fn compute_eta(kappa: u64, u: usize, v: usize) -> f64 {
    let p = 1.0 - (kappa as f64) / 160.0;
    let u = u as f64;
    let v = v as f64;
    let term1 = v - u * p;
    let term2 = (2.0 * u * p * (1.0 - p)).sqrt();

    0.5 * erfc(term1 / term2).unwrap().0
}

pub fn compute_q(kappa: u64, u: usize, r: usize) -> f64 {
    let p = 1.0 - (kappa as f64) / 160.0;
    let term1 = (1.0 - p).powf((u - r) as f64);
    let term2 = approx_binomial(u - 1, r - 1);

    let u = u as f64;
    let r = r as f64;

    let term3 = hyper_2f1(u - r, 1.0 - r, 1.0 + u - r, 1.0 - p).unwrap().0;

    term1 * term2 * term3
}

pub fn approx_binomial(n: usize, k: usize) -> f64 {
    let n = n as f64;
    let k = k as f64;
    let pi = std::f64::consts::PI;

    let term1 = (n / (2.0 * pi * k * (n - k))).sqrt();
    let term2 = n.powf(n) / (k.powf(k) * (n - k).powf(n - k));

    term1 * term2
}

pub fn compute_delta_u(eta0: f64, kappa: u64, u: usize, v: usize) -> usize {
    use statrs::function::erf::erfc_inv;

    let p = 1.0 - (kappa as f64) / 160.0;
    let alpha = erfc_inv(2.0 * eta0);

    ((u as f64)
        - (alpha
            * (alpha * (1.0 - p)
                + ((1.0 - p) * (alpha * alpha * (1.0 - p) + 2.0 * (v as f64))).sqrt())
            + (v as f64))
            / p) as usize
}

/// Computes 2F1 and returns `Some(value, error estimate)` on success
//
// Note: this relies on GSL, which may need to be installed:
//      sudo apt install libgsl0-dev
// or   brew install gsl
//
// In case of failure, debug information is printed out
pub fn hyper_2f1(a: f64, b: f64, c: f64, x: f64) -> Option<(f64, f64)> {
    use rgsl::{hypergeometric::hyperg_2F1_e, Value};

    let (code, res) = hyperg_2F1_e(a, b, c, x);
    match code {
        Value::Success => Some((res.val, res.err)),
        _ => {
            dbg!(code);
            None
        }
    }
}

/// Computes erfc(x) and returns `Some(value, error estimate)` on success
//
// Note: this relies on GSL, which may need to be installed:
//      sudo apt install libgsl0-dev
// or   brew install gsl
//
// In case of failure, debug information is printed out
pub fn erfc(x: f64) -> Option<(f64, f64)> {
    use rgsl::{error::erfc_e, Value};

    let (code, res) = erfc_e(x);
    match code {
        Value::Success => Some((res.val, res.err)),
        _ => {
            dbg!(code);
            None
        }
    }
}
