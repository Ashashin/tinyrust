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

    #[test]
    fn draw_acceptance() -> Result<(), Report> {
        use plotters::prelude::*;
        use stats::compute_q;
        use std::time::Instant;
        use vm::{validate_hash, InstrumentedVM};

        let u = 1000000;
        let delta = 0.1;
        let u_max = ((1.0 + delta) * u as f64) as usize;
        let u_min = ((1.0 - delta) * u as f64) as usize;

        let kappa_min = 144;
        let kappa_max = 159;
        let kappa_num = 5;
        let get_kappa = |i: usize| (kappa_max - kappa_min) * i / (kappa_num - 1) + kappa_min;

        let mut data: Vec<Vec<(usize, f64)>> = vec![vec![Default::default(); u_max]; kappa_num];

        let start = Instant::now();
        let mut vm = InstrumentedVM::new(String::from("../assets/collatz_v0.tr"))?;

        // Accumulator for the valid number of hashes
        let mut acc: Vec<usize> = vec![0; kappa_num];

        // Create data points form vm run
        (0..u_max).for_each(|i| {
            let h = vm.run(i).unwrap().hash;

            // Apply each hash to a kappa
            data.iter_mut().enumerate().for_each(|(k, v)| {
                let kappa = get_kappa(k);
                if validate_hash(&h, kappa) {
                    acc[k] += 1;
                }
                v[i] = (i, compute_q(kappa as u64, u, acc[k]));
            })
        });

        println!("Got traces in: {:?}", start.elapsed());

        // Graph part
        let root = BitMapBackend::new("graoh.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;

        // Two parts: one global [0, (1+delta)*u] and another localised [(1-delta)*u, (1+delta)*u]
        let (upper, lower) = root.split_vertically(384);

        let mut chart1 = ChartBuilder::on(&upper)
            .caption(
                format!(
                    "Acceptance for a claim of U={} for the collatz conjecture",
                    u
                ),
                ("sans-serif", 20).into_font(),
            )
            .margin(30)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(0..u_max, 0.0..1.0)?;

        let mut chart2 = ChartBuilder::on(&lower)
            .caption(
                format!(
                    "Acceptance for a claim of U={} for the collatz conjecture (±{}U area)",
                    u, delta
                ),
                ("sans-serif", 20).into_font(),
            )
            .margin(30)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(u_min..u_max, 0.0..1.0)?;

        chart1
            .configure_mesh()
            .x_desc("Actual range tested u")
            .y_desc("Acceptance q")
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        chart2
            .configure_mesh()
            .x_desc("Actual range tested u")
            .y_desc("Acceptance q")
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        data.into_iter().enumerate().for_each(|(k, d)| {
            let local_start = Instant::now();
            let kappa = get_kappa(k);

            chart1
                .draw_series(LineSeries::new(d.clone().into_iter(), &Palette99::pick(k)))
                .unwrap()
                .label(format!("Kappa = 2^{}", kappa))
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], &Palette99::pick(k))
                });

            chart2
                .draw_series(LineSeries::new(
                    d.into_iter().skip(u_min),
                    &Palette99::pick(k),
                ))
                .unwrap()
                .label(format!("Kappa = 2^{}", kappa))
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], &Palette99::pick(k))
                });

            println!(
                "Printed for kappa = 2^{} in {:?}",
                kappa,
                local_start.elapsed()
            );
        });

        // Delimit the value U
        chart1.draw_series([PathElement::new(vec![(u, 0.0), (u, 1.0)], BLACK)])?;
        chart2.draw_series([PathElement::new(vec![(u, 0.0), (u, 1.0)], BLACK)])?;

        chart1
            .configure_series_labels()
            .position(SeriesLabelPosition::MiddleLeft)
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        chart2
            .configure_series_labels()
            .position(SeriesLabelPosition::MiddleLeft)
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        Ok(())
    }
}
