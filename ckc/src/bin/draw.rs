use color_eyre::Report;
use plotters::prelude::*;
use structopt::StructOpt;

use std::{path::PathBuf, time::Instant};

use ckc::get_data;

/// Command line options
#[derive(Debug, StructOpt)]
struct Opt {
    /// Program file
    #[structopt(parse(from_os_str))]
    program: PathBuf,

    /// Claimed value
    #[structopt(default_value = "10000")]
    u: usize,

    /// Delta u range
    #[structopt(short, long, default_value = "0.1")]
    delta: f64,
}

fn main() -> Result<(), Report> {
    let opt = Opt::from_args();

    let u = opt.u;
    let delta = opt.delta;

    let u_max = ((1.0 + delta) * u as f64) as usize;
    let u_min = ((1.0 - delta) * u as f64) as usize;

    let data = get_data(opt.program, u, u_max)?;

    // Graph part
    let root = BitMapBackend::new("graph.png", (1024, 768)).into_drawing_area();
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

    data.into_iter().enumerate().for_each(|(k, (kappa, d))| {
        let local_start = Instant::now();

        chart1
            .draw_series(LineSeries::new(
                d.clone().into_iter().enumerate(),
                &Palette99::pick(k),
            ))
            .unwrap()
            .label(format!("Kappa = 2^{}", kappa))
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &Palette99::pick(k)));

        chart2
            .draw_series(LineSeries::new(
                d.into_iter().enumerate().skip(u_min),
                &Palette99::pick(k),
            ))
            .unwrap()
            .label(format!("Kappa = 2^{}", kappa))
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &Palette99::pick(k)));

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
