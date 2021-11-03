mod proof;
mod prover;
mod stats;
mod verifier;
mod vm;

pub use proof::{ProofParams, ProofStrategy};
pub use prover::Prover;
pub use verifier::Verifier;

#[cfg(test)]
mod tests {
    use crate::*;
    use color_eyre::Report;

    #[test]
    fn run_prover_and_verifier() -> Result<(), Report> {
        let params = ProofParams::new(
            "../assets/collatz_v0.tr",
            1..1000,
            0,
            155,
            1000,
            ProofStrategy::BestEffort,
        );
        let prover = Prover::new(params);

        // Get proof
        let proof = prover.obtain_proof()?;
        let verifier = Verifier::new(proof);

        // Check proof
        let result = verifier.check_proof();

        result.display();

        Ok(())
    }
}
