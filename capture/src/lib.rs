pub mod capture;
pub use capture::start_capture;
pub use capture::{PacketFeatures, ClassifierOutput, CaptureConfig};

pub struct TrafficClassifier;

impl TrafficClassifier {
    pub fn new() -> Self { Self }
}