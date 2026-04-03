//! ML classification client for connecting to Python ML server.
//! 
//! Provides:
//! - HTTP client for talking to ML inference server
//! - Fallback to rule-based classification when server unavailable
//! - Batch classification support

use serde::{Deserialize, Serialize};
use std::time::Duration;
use capture::classify_port;

/// ML classification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLClassification {
    /// Class ID from model
    pub class_id: u8,
    /// Class name (e.g., "HTTP", "HTTPS")
    pub class_name: String,
    /// Confidence score [0.0, 1.0]
    pub confidence: f32,
}

/// Client for ML inference server.
pub struct MLClient {
    server_url: String,
    client: reqwest::Client,
    use_ml: bool,
}

impl MLClient {
    /// Create new ML client.
    pub fn new(server_url: &str, use_ml: bool) -> Self {
        Self {
            server_url: server_url.to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
            use_ml,
        }
    }
    
    /// Classify packet features using ML model.
    /// 
    /// Returns ML classification if server available, otherwise None.
    pub async fn classify(&self, features: &[f32]) -> Option<MLClassification> {
        if !self.use_ml {
            return None;
        }
        
        let url = format!("{}/classify", self.server_url);
        let payload = serde_json::json!({
            "features": features
        });
        
        match self.client.post(&url)
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    response.json::<MLClassification>().await.ok()
                } else {
                    eprintln!("ML server returned error: {}", response.status());
                    None
                }
            }
            Err(e) => {
                eprintln!("ML server connection failed: {}", e);
                None
            }
        }
    }
    
    /// Check if ML server is available.
    pub async fn is_available(&self) -> bool {
        if !self.use_ml {
            return false;
        }
        
        let url = format!("{}/health", self.server_url);
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

/// ML Classification mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClassificationMode {
    /// Use rule-based (port-based) classification
    RuleBased,
    /// Use ML model via server
    ML,
}

/// Unified classifier that can use either rule-based or ML.
pub struct Classifier {
    mode: ClassificationMode,
    ml_client: Option<MLClient>,
}

impl Classifier {
    /// Create classifier with ML server.
    pub fn new(ml_server_url: Option<&str>, use_ml: bool) -> Self {
        let mode = if use_ml && ml_server_url.is_some() {
            ClassificationMode::ML
        } else {
            ClassificationMode::RuleBased
        };
        
        let ml_client = ml_server_url.map(|url| MLClient::new(url, use_ml));
        
        Self { mode, ml_client }
    }
    
    /// Classify packet features.
    /// 
    /// Uses ML if available, otherwise falls back to rule-based.
    pub async fn classify(&self, features: &capture::PacketFeatures) -> (String, f32) {
        // Try ML first if available
        if let Some(ref client) = self.ml_client {
            if self.mode == ClassificationMode::ML {
                let feature_vec = features.to_vector();
                if let Some(ml_result) = client.classify(&feature_vec).await {
                    return (ml_result.class_name, ml_result.confidence);
                }
            }
        }
        
        // Fall back to rule-based
        capture::classify_port(features.dst_port)
    }
    
    /// Check ML server availability.
    pub async fn check_ml(&self) -> bool {
        if let Some(ref client) = self.ml_client {
            client.is_available().await
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_classifier_fallback() {
        // Create without ML server - should use rule-based
        let classifier = Classifier::new(None, false);
        
        // This would test rule-based classification if we had features
        assert_eq!(classifier.mode, ClassificationMode::RuleBased);
    }
}