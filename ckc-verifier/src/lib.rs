use ckc_prover::{run_instrumented_vm, validate_hash, Proof, ProofStrategy, ProverParams};

pub struct Verifier {}
pub struct ProofReport {
    proof: Proof,
    valid_vset: Vec<usize>,
    eta: f64,
    q: f64,
}

impl Verifier {
    pub fn check_proof(proof: Proof) -> ProofReport {
        match proof.params.strategy {
            ProofStrategy::BestEffort => Self::check_proof_best_effort(proof),
            _ => unimplemented!("Unsupported proof strategy: {:?}", proof.params.strategy),
        }
    }

    fn check_proof_best_effort(proof: Proof) -> ProofReport {
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let valid_vset = Self::validate_vset(&proof.vset, &proof.params);
        let v = valid_vset.len();

        let eta = compute_eta(kappa, u, v);
        let q = compute_q(kappa, u, v);

        ProofReport {
            proof,
            valid_vset,
            eta,
            q,
        }
    }

    fn validate_vset(vset: &Vec<usize>, params: &ProverParams) -> Vec<usize> {
        let mut new_vset = vec![];

        for &i in vset {
            if params.input_domain.contains(&i) && vset.len() > params.v {
                // TODO: separate actual failure from the discovery of a counter-example ?
                let succes = match run_instrumented_vm(params.program_file.clone(), i) {
                    Ok(res) => {
                        res.output == params.expected_output
                            && validate_hash(res.hash, params.kappa as usize)
                    }
                    Err(_e) => false,
                };

                if succes {
                    new_vset.push(i);
                }
            }
        }
        new_vset
    }
}

fn compute_eta(kappa: u64, u: usize, v: usize) -> f64 {
    let p = 1.0 - (kappa as f64) / 160.0;
    let u = u as f64;
    let v = v as f64;
    let term1 = v - u * p;
    let term2 = (2.0 * u * p * (1.0 - p)).sqrt();

    0.5 * erfc(term1 / term2).unwrap().0
}

fn compute_q(kappa: u64, u: usize, r: usize) -> f64 {
    let p = 1.0 - (kappa as f64) / 160.0;
    let term1 = (1.0 - p).powf((u - r) as f64);
    let term2 = approx_binomial(u - 1, r - 1);

    let u = u as f64;
    let r = r as f64;

    let term3 = hyper_2f1(u - r, 1.0 - r, 1.0 + u - r, 1.0 - p).unwrap().0;

    term1 * term2 * term3
}

fn approx_binomial(n: usize, k: usize) -> f64 {
    let n = n as f64;
    let k = k as f64;
    let pi = std::f64::consts::PI;

    let term1 = (n / (2.0 * pi * k * (n - k))).sqrt();
    let term2 = n.powf(n) / (k.powf(k) * (n - k).powf(n - k));

    term1 * term2
}

/// Computes 2F1 and returns `Some(value, error estimate)` on success
//
// Note: this relies on GSL, which may need to be installed:
//      sudo apt install libgsl0-dev
// or   brew install gsl
//
// In case of failure, debug information is printed out
fn hyper_2f1(a: f64, b: f64, c: f64, x: f64) -> Option<(f64, f64)> {
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
