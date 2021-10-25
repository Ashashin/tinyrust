use ckc_prover::Proof;

pub struct Verifier {}
pub struct ProofReport {}

impl Verifier {
    pub fn check_proof(proof: Proof) -> ProofReport {
        todo!()
    }
}

/// Computes 2F1 and returns `Some(value, error estimate)` on success
//
// Note: this relies on GSL, which may need to be installed:
//      sudo apt install libgsl0-dev
// or   brew install gsl
//
// In case of failure, debug information is printed out
fn hyper_2F1(a: f64, b: f64, c: f64, x: f64) -> Option<(f64, f64)> {
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
fn erfc(x: f64) -> Option<(f64, f64)> {
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
