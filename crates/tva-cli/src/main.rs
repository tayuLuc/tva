#![forbid(unsafe_code)]

//! tva CLI — analyze video files from the command line.
//! Machine-parseable JSON/CSV output for AI agents and scripts.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tva", about = "Temporal Video Analyzer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Analyze video file(s)
    Analyze {
        /// Input video file(s) or glob
        input: Vec<PathBuf>,

        /// Output format (json, csv)
        #[arg(long, default_value = "json")]
        format: String,

        /// Specific metrics (comma-separated)
        #[arg(long)]
        metrics: Option<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Render overlay video with FPS graph
    Overlay {
        /// Input video
        input: PathBuf,
        /// Output video
        #[arg(short, long)]
        output: PathBuf,
        /// Codec (h264, h265)
        #[arg(long, default_value = "h264")]
        codec: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Analyze { input, format, metrics, output } => {
            // ponytail: stub — full impl in phase 3
            let _ = (format, metrics, output);
            eprintln!("tva analyze: {} file(s)", input.len());
        }
        Command::Overlay { input, output, codec } => {
            // ponytail: stub — full impl in phase 7
            let _ = codec;
            eprintln!("tva overlay: {} -> {}", input.display(), output.display());
        }
    }
}
