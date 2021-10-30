pub mod prover;
mod stats;
pub mod verifier;

#[cfg(test)]
mod tests {
    use crate::*;
    use color_eyre::Report;
    use tinyvm::{parser::Parser, run_vm};

    #[test]
    fn run_fibo() -> Result<(), Report> {
        let update_hash = |_: &[u8]| {};

        let mut vm = Parser::load_program(&String::from("../assets/fib.tr"))?;
        let result = run_vm(&mut vm, vec![39], update_hash)?;
        println!("Result = {}", result);

        assert_eq!(result, 63245986);
        Ok(())
    }

    #[test]
    fn run_fib_with_instrumentation() -> Result<(), Report> {
        let mut vm = prover::InstrumentedVM::new(&String::from("../assets/fib.tr"))?;
        let result = vm.run(39)?;
        println!("Result = {:?}", result);

        let expected_output = 63245986;

        assert_eq!(result.output, expected_output);

        Ok(())
    }

    #[test]
    fn run_collatz_with_instrumentation() -> Result<(), Report> {
        let mut vm = prover::InstrumentedVM::new(&String::from("../assets/collatz_v0.tr"))?;

        let result = vm.run(39)?;
        println!("Result = {:?}", result);

        let expected_output = 0;

        assert_eq!(result.output, expected_output);

        Ok(())
    }

    #[test]
    fn run_prover_and_verifier() -> Result<(), Report> {
        let prover = prover::Prover::new(prover::ProverParams {
            program_file: String::from("../assets/collatz_v0.tr"),
            input_domain: 1..1000,
            expected_output: 0,
            strategy: prover::ProofStrategy::BestEffort,
            kappa: 8,
            v: 1000,
        });

        // Get proof
        let proof = prover.obtain_proof()?;

        // Check proof
        let epsilon = 0.99;
        let result = verifier::Verifier::check_proof(proof, epsilon);

        result.display();

        Ok(())
    }

    #[test]
    fn report_display() {
        let fake_proof = verifier::ProofReport {
            proof: prover::Proof {
                vset: vec![],
                params: prover::ProverParams {
                    program_file: String::from("none.txt"),
                    input_domain: 42..69,
                    expected_output: 33,
                    kappa: 12,
                    v: 3,
                    strategy: prover::ProofStrategy::BestEffortAdaptive,
                },
                extended_domain: None,
            },
            eta: 0.4,
            q: 0.6,
            valid: false,
        };

        fake_proof.display();

        println!("JSON: {}", fake_proof.export());
    }
}
