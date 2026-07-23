use std::fs;
use std::path::Path;

/// Golden test: Report JSON schema backward compatibility.
/// Старые файлы должны десериализоваться новым кодом.
#[test]
fn golden_empty_report() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/empty_report.json");
    let json = fs::read_to_string(path).expect("golden file exists");
    let report: tva_core::Report = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.meta.fps, 0.0);
    assert!(report.tears.is_none()); // additive field → None
    assert!(report.resolution.is_none());
}

/// Workspace-level test: check that core types are re-exported.
#[test]
fn pixel_buffer_from_bytes() {
    let buf = tva_core::PixelBuffer::new(vec![0u8; 12], 2, 2).unwrap();
    assert_eq!(buf.width(), 2);
    assert_eq!(buf.pixel_count(), 4);
}
