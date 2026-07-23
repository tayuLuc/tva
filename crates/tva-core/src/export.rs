use crate::error::Result;
use crate::report::Report;
use std::path::Path;

/// Serialize report as pretty JSON.
pub fn to_json(report: &Report) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

/// Write per-frame metrics to CSV.
pub fn to_csv(report: &Report, out: &Path) -> Result<()> {
    let mut wtr = csv::Writer::from_path(out)?;
    wtr.write_record(&["container_frame", "unique_frame", "streak_length", "real_frame_time_ms", "instantaneous_fps"])?;
    for f in &report.frames {
        wtr.write_record(&[
            f.container_frame.to_string(),
            f.unique_frame.to_string(),
            f.streak_length.to_string(),
            format!("{:.4}", f.real_frame_time_ms),
            format!("{:.4}", f.instantaneous_fps),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}
