#[derive(Debug, Clone)]
pub enum AnalysisEvent {
    Progress { frame: u64, total: u64 },
    DuplicateFound { frame: u64, streak: u32 },
    TearDetected { frame: u64, position: f32 },
    SecondComplete { second: u32, unique_frames: u32 },
}

pub trait EventSink {
    fn on_event(&mut self, event: AnalysisEvent);
}

pub struct NullSink;

impl EventSink for NullSink {
    fn on_event(&mut self, _event: AnalysisEvent) {}
}
