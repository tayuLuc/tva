use std::fs;
use std::path::Path;
use tva_core::{
    adapters::{identity_smoother::IdentitySmoother, image_compare::SsimComparator},
    config::PipelineConfig,
    events::NullSink,
    pipeline, Report,
};

#[test]
fn golden_empty_report() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/empty_report.json");
    let json = fs::read_to_string(path).expect("golden file exists");
    let report: Report = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.meta.fps, 0.0);
    assert!(report.tears.is_none());
    assert!(report.resolution.is_none());
}

#[test]
fn pixel_buffer_from_bytes() {
    let buf = tva_core::PixelBuffer::new(vec![0u8; 12], 2, 2).unwrap();
    assert_eq!(buf.width(), 2);
    assert_eq!(buf.pixel_count(), 4);
}

#[cfg(feature = "decode-images")]
#[test]
fn fixture_duplicate_detection() {
    use tva_core::adapters::image_seq::ImageSeqDecoder;
    use tva_core::traits::{FrameDecoder, Smoother};

    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/frames");
    let mut decoder = ImageSeqDecoder::open(&dir, Some(30.0)).unwrap();
    let comparator = SsimComparator;
    let smoother: Box<dyn Smoother> = Box::new(IdentitySmoother);
    let config = PipelineConfig { duplicate_threshold: 0.98, detect_tears: true, ..Default::default() };

    let mut sink = NullSink;
    let report: Report = pipeline::analyze(&mut decoder, &config, &comparator, smoother.as_ref(), &mut sink).unwrap();

    // 5 total frames, 3 unique (frames 1+2 are dupes, 4+5 are dupes)
    assert_eq!(report.summary.total_container_frames, 5);
    assert_eq!(report.summary.total_unique_frames, 3);
    assert_eq!(report.summary.duplicate_count, 2);
}
