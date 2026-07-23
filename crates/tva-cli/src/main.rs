#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tva_core::adapters::image_compare::metric_by_name;
use tva_core::{config::PipelineConfig, events::NullSink, pipeline, traits::Smoother, Report};

#[derive(Parser)]
#[command(name = "tva", version, about = "Temporal Video Analyzer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Analyze {
        input: PathBuf,
        #[arg(long, default_value = "json")]
        format: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        fps: Option<f64>,
        /// Comparison metric: ssim, hybrid, mad
        #[arg(long, default_value = "ssim")]
        metric: String,
        /// Duplicate threshold in the metric's native scale.
        /// If omitted, the metric's own default is used.
        #[arg(long)]
        threshold: Option<f64>,
        #[arg(long)]
        no_tears: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> tva_core::Result<()> {
    match cli.command {
        Command::Analyze { input, format, output, fps, metric, threshold, no_tears } => {
            let mut decoder: Box<dyn tva_core::traits::FrameDecoder> = if input.is_dir() {
                Box::new(tva_core::adapters::image_seq::ImageSeqDecoder::open(&input, fps)?)
            } else {
                return Err(tva_core::TvaError::Decode(
                    "video files require decode-ffmpeg feature; use a directory of PNG/JPG frames instead".into(),
                ));
            };

            let spec = metric_by_name(&metric)?;
            let dup_threshold = threshold.unwrap_or(spec.default_threshold);

            #[cfg(feature = "smooth-savgol")]
            let smoother: Box<dyn Smoother> =
                Box::new(tva_core::adapters::savgol::SavgolSmoother { window: 21, polyorder: 3 });
            #[cfg(not(feature = "smooth-savgol"))]
            let smoother: Box<dyn Smoother> = Box::new(tva_core::adapters::identity_smoother::IdentitySmoother);

            let config =
                PipelineConfig { duplicate_threshold: dup_threshold, detect_tears: !no_tears, ..Default::default() };

            let mut sink = NullSink;
            let report: Report =
                pipeline::analyze(decoder.as_mut(), &config, spec.comparator.as_ref(), smoother.as_ref(), &mut sink)?;

            match format.as_str() {
                "json" => {
                    let json = tva_core::export::to_json(&report)?;
                    write_output(&output, &json)?;
                }
                "csv" => {
                    if let Some(path) = &output {
                        tva_core::export::to_csv(&report, path)?;
                    } else {
                        let mut w = csv::Writer::from_writer(std::io::stdout());
                        w.write_record([
                            "container_frame",
                            "unique_frame",
                            "streak_length",
                            "real_frame_time_ms",
                            "instantaneous_fps",
                        ])
                        .map_err(|e| tva_core::TvaError::Csv(e.to_string()))?;
                        for f in &report.frames {
                            w.write_record(&[
                                f.container_frame.to_string(),
                                f.unique_frame.to_string(),
                                f.streak_length.to_string(),
                                format!("{:.4}", f.real_frame_time_ms),
                                format!("{:.4}", f.instantaneous_fps),
                            ])
                            .map_err(|e| tva_core::TvaError::Csv(e.to_string()))?;
                        }
                        w.flush().map_err(|e| tva_core::TvaError::Csv(e.to_string()))?;
                    }
                }
                other => {
                    return Err(tva_core::TvaError::Decode(format!("unknown format: {other} (expected json or csv)")))
                }
            }

            let s = &report.summary;
            eprintln!(
                "analyzed {} frames: {} unique, {} duplicates, {} tears | avg {:.1} fps, 1% low {:.1}, P99 {:.1} ms",
                s.total_container_frames,
                s.total_unique_frames,
                s.duplicate_count,
                s.tear_count,
                s.avg_fps,
                s.fps_1_low,
                s.p99_frame_time_ms,
            );

            Ok(())
        }
    }
}

fn write_output(path: &Option<PathBuf>, content: &str) -> tva_core::Result<()> {
    match path {
        Some(p) => {
            std::fs::write(p, content)?;
            eprintln!("written to {}", p.display());
        }
        None => print!("{content}"),
    }
    Ok(())
}
