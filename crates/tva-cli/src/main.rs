#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command as SystemCommand;
use tva_core::adapters::{
    ffmpeg_native::FfmpegNativeDecoder, identity_smoother::IdentitySmoother, image_compare::metric_by_name,
    image_seq::ImageSeqDecoder,
};
use tva_core::{
    config::PipelineConfig,
    degradation::{compare_sources, DegradationConfig},
    events::NullSink,
    pipeline,
    traits::{FrameDecoder, Smoother},
    Report,
};

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

    /// Compare two versions of the same video (e.g. before/after re-encoding)
    Compare {
        /// Source A: directory of frames (original)
        a: PathBuf,
        /// Source B: directory of frames (compressed / re-encoded)
        b: PathBuf,
        /// Comparison metric: ssim, hybrid, mad
        #[arg(long, default_value = "ssim")]
        metric: String,
        /// Max timestamp drift for frame matching, ms (VFR sources: raise this)
        #[arg(long)]
        drift_ms: Option<f64>,
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Downscale a video to its native resolution (calls external ffmpeg)
    Descale {
        /// Input video file
        input: PathBuf,
        /// Target: height (480, 480p, 720p) or exact WxH (854x480)
        #[arg(long)]
        to: String,
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
        /// Video codec
        #[arg(long, default_value = "libx264")]
        codec: String,
        /// Constant rate factor (lower = higher quality)
        #[arg(long, default_value_t = 18)]
        crf: u8,
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
            let mut decoder = open_decoder(&input, fps)?;

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
        Command::Compare { a, b, metric, drift_ms, output } => {
            let spec = metric_by_name(&metric)?;
            let mut dec_a = open_decoder(&a, None)?;
            let mut dec_b = open_decoder(&b, None)?;

            let cfg = match drift_ms {
                Some(d) => DegradationConfig { max_time_drift_ms: d },
                None => DegradationConfig::default(),
            };

            let mut sink = NullSink;
            let report = compare_sources(dec_a.as_mut(), dec_b.as_mut(), spec.comparator.as_ref(), &cfg, &mut sink)?;

            let json = serde_json::to_string_pretty(&report)?;
            write_output(&output, &json)?;

            let s = &report.summary;
            if let Some(m) = &report.size_mismatch {
                eprintln!(
                    "warning: resolution mismatch {}x{} vs {}x{} — no pixel comparison performed",
                    m.a.0, m.a.1, m.b.0, m.b.1
                );
            }
            eprintln!(
                "compared {} pairs ({} dropped) | mean similarity {:.3}, min {:.3}, quality drop {:.1}%",
                s.pairs_compared, s.pairs_dropped, s.mean_similarity, s.min_similarity, s.quality_drop_pct,
            );

            Ok(())
        }
        Command::Descale { input, to, output, codec, crf } => {
            if !input.is_file() {
                return Err(tva_core::TvaError::Decode(format!("input not found: {}", input.display())));
            }
            let target = parse_target(&to)?;
            let vf = scale_filter(target);

            let status = SystemCommand::new("ffmpeg")
                .arg("-y")
                .arg("-i")
                .arg(&input)
                .arg("-vf")
                .arg(&vf)
                .arg("-c:v")
                .arg(&codec)
                .arg("-crf")
                .arg(crf.to_string())
                .arg("-c:a")
                .arg("copy")
                .arg(&output)
                .status()
                .map_err(|e| {
                    tva_core::TvaError::Decode(format!("ffmpeg not found in PATH ({e}); install ffmpeg to use descale"))
                })?;

            if !status.success() {
                return Err(tva_core::TvaError::Decode(format!("ffmpeg exited with {status}")));
            }
            eprintln!("descaled {} -> {} ({vf})", input.display(), output.display());
            Ok(())
        }
    }
}

/// Open a decoder: directory -> ImageSeqDecoder, file -> native libav decoder.
fn open_decoder(path: &Path, fps: Option<f64>) -> tva_core::Result<Box<dyn FrameDecoder>> {
    if path.is_dir() {
        Ok(Box::new(ImageSeqDecoder::open(path, fps)?))
    } else {
        Ok(Box::new(FfmpegNativeDecoder::open(path)?))
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

#[derive(Debug, Clone, Copy)]
enum ScaleTarget {
    Both(u32, u32),
    Height(u32),
}

fn parse_target(s: &str) -> tva_core::Result<ScaleTarget> {
    let s = s.trim().trim_end_matches(|c| c == 'p' || c == 'P');
    if s.is_empty() {
        return Err(tva_core::TvaError::BadScale(String::new()));
    }
    if let Some((w, h)) = s.split_once(['x', 'X', ':']) {
        let w = w.trim().parse::<u32>().map_err(|_| tva_core::TvaError::BadScale(s.into()))?;
        let h = h.trim().parse::<u32>().map_err(|_| tva_core::TvaError::BadScale(s.into()))?;
        Ok(ScaleTarget::Both(w, h))
    } else {
        let h = s.parse::<u32>().map_err(|_| tva_core::TvaError::BadScale(s.into()))?;
        Ok(ScaleTarget::Height(h))
    }
}

fn scale_filter(t: ScaleTarget) -> String {
    match t {
        ScaleTarget::Both(w, h) => format!("scale={w}:{h}:flags=lanczos"),
        ScaleTarget::Height(h) => format!("scale=-2:{h}:flags=lanczos"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_only() {
        assert!(matches!(parse_target("480").unwrap(), ScaleTarget::Height(480)));
        assert!(matches!(parse_target("480p").unwrap(), ScaleTarget::Height(480)));
        assert!(matches!(parse_target(" 720P ").unwrap(), ScaleTarget::Height(720)));
    }

    #[test]
    fn both_separators() {
        assert!(matches!(parse_target("854x480").unwrap(), ScaleTarget::Both(854, 480)));
        assert!(matches!(parse_target("854X480").unwrap(), ScaleTarget::Both(854, 480)));
        assert!(matches!(parse_target("854:480").unwrap(), ScaleTarget::Both(854, 480)));
    }

    #[test]
    fn garbage_is_err() {
        assert!(parse_target("foo").is_err());
        assert!(parse_target("x480").is_err());
        assert!(parse_target("").is_err());
    }

    #[test]
    fn filter_strings() {
        assert_eq!(scale_filter(ScaleTarget::Height(480)), "scale=-2:480:flags=lanczos");
        assert_eq!(scale_filter(ScaleTarget::Both(854, 480)), "scale=854:480:flags=lanczos");
    }
}
