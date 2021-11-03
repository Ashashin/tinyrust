use statrs::{
    distribution::{DiscreteCDF, NegativeBinomial},
    function::erf::{erfc, erfc_inv},
};

/// Compute the probability of getting an acceptable witness set
pub fn compute_eta(kappa: u64, u: usize, v: usize) -> f64 {
    let p = derive_p(kappa);
    let u = u as f64;
    let v = v as f64;
    let term1 = v - u * p;
    let term2 = (2.0 * u * p * (1.0 - p)).sqrt();

    0.5 * erfc(term1 / term2)
}

/// Compute the probability of validity of the proof
pub fn compute_q(kappa: u64, u: usize, r: usize) -> f64 {
    if u < 1 || r < 1 {
        return 0.0;
    }

    let p = derive_p(kappa);
    let d = (u - r + 1).try_into().unwrap();
    let nb = NegativeBinomial::new(r as f64, p).unwrap();

    1.0 - nb.cdf(d)
}

/// Compute the extra number of step required to attain the eta0 threshold
pub fn compute_delta_u(eta0: f64, kappa: u64, u: usize, v: usize) -> usize {
    let p = derive_p(kappa);
    let alpha = erfc_inv(2.0 * eta0);

    ((u as f64)
        - alpha.mul_add(
            alpha.mul_add(
                1.0 - p,
                ((1.0 - p) * (alpha * alpha).mul_add(1.0 - p, 2.0 * (v as f64))).sqrt(),
            ),
            v as f64,
        ) / p) as usize
}

/// Compute the minimal number of witness to attain the eta0 threshold
pub fn compute_v_min(eta0: f64, kappa: u64, u: usize) -> usize {
    let p = derive_p(kappa);
    let alpha = erfc_inv(2.0 * eta0);
    let beta = u as f64 * p;

    (beta * (1.0 - p)).sqrt().mul_add(alpha, beta) as usize
}

/// Derivee the probability from the kappa value
fn derive_p(kappa: u64) -> f64 {
    (kappa as f64 - 160.0).exp2()
}
