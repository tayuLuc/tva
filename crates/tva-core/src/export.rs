//! Export analysis results to JSON, CSV, overlay video.

use crate::pipeline::Report;
use std::path::Path;

/// Serialize report as pretty JSON.
pub fn to_json(report: &Report) -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string_pretty(report)?)
}

/// Write per-frame metrics to CSV.
pub fn to_csv(report: &Report, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(out)?;
    wtr.write_record(&[
        "frame",
        "pts_us",
        "delta_us",
        "duplicate",
        "tear_line",
        "delta_cielab",
    ])?;
    for f in &report.frames {
        wtr.write_record(&[
            f.frame_num.to_string(),
            f.pts_us.to_string(),
            f.delta_us.to_string(),
            f.duplicate.to_string(),
            f.tear_line.map(|t| t.to_string()).unwrap_or_default(),
            format!("{:.6}", f.delta_cielab),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}
