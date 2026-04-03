pub mod capture;
pub use capture::start_capture;
pub use capture::{PacketFeatures, ClassifierOutput, CaptureConfig, classify_port};

pub struct TrafficClassifier;

impl TrafficClassifier {
    pub fn new() -> Self { Self }
}