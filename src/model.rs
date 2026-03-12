//! AI Model client for intelligent input prediction
//!
//! Supports both remote API models (like 0.8B models) and local models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::config::ModelConfig;

/// Model client trait for different backends
pub trait ModelBackend: Send + Sync {
    /// Predict next characters/phrases given input context
    fn predict(&self, context: &str, input: &str) -> Vec<PredictionResult>;
    
    /// Check if model is available
    fn is_available(&self) -> bool;
}

/// Prediction result from the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// The predicted text
    pub text: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Source of the prediction
    pub source: PredictionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PredictionSource {
    /// From AI model
    AIModel,
    /// From dictionary
    Dictionary,
    /// From user history
    UserHistory,
    /// From fuzzy matching
    FuzzyMatch,
    /// Built-in pinyin mapping
    BuiltIn,
}

/// Remote model client (HTTP API based)
pub struct RemoteModelClient {
    config: ModelConfig,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, Vec<PredictionResult>>>>,
}

impl RemoteModelClient {
    pub fn new(config: ModelConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();
        
        Self {
            config,
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Build prompt for the model
    fn build_prompt(&self, context: &str, input: &str) -> String {
        format!(
            "你是一个中文输入法助手。根据用户的输入上下文和当前拼音输入，预测用户可能想输入的中文。\n\
             上下文：{}\n\
             当前拼音输入：{}\n\
             请直接输出最可能的5个中文候选词，每行一个，不要有其他内容：",
            context, input
        )
    }
    
    /// Call remote API for prediction
    pub async fn predict_async(&self, context: &str, input: &str) -> Vec<PredictionResult> {
        // Check cache first
        let cache_key = format!("{}|{}", context, input);
        {
            let cache = self.cache.read();
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }
        
        let prompt = self.build_prompt(context, input);
        
        let request = ChatCompletionRequest {
            model: self.config.model_name.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };
        
        match self.call_api(&request).await {
            Ok(response) => {
                let results = self.parse_response(&response);
                
                // Cache results
                if self.config.enable_cache {
                    let mut cache = self.cache.write();
                    if cache.len() >= self.config.cache_size {
                        // Simple cache eviction: remove first entry
                        if let Some(first_key) = cache.keys().next().cloned() {
                            cache.remove(&first_key);
                        }
                    }
                    cache.insert(cache_key, results.clone());
                }
                
                results
            }
            Err(e) => {
                log::warn!("Model API call failed: {}", e);
                vec![]
            }
        }
    }
    
    async fn call_api(&self, request: &ChatCompletionRequest) -> Result<ChatCompletionResponse, String> {
        let url = format!("{}/chat/completions", self.config.api_endpoint);
        
        let mut req = self.client.post(&url).json(request);
        
        if let Some(ref api_key) = self.config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()));
        }
        
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
    
    fn parse_response(&self, response: &ChatCompletionResponse) -> Vec<PredictionResult> {
        let content = response
            .choices
            .first()
            .map(|c| c.message.content.as_str())
            .unwrap_or("");
        
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .enumerate()
            .map(|(i, line)| PredictionResult {
                text: line.trim().to_string(),
                confidence: 1.0 - (i as f32 * 0.1),
                source: PredictionSource::AIModel,
            })
            .collect()
    }
}

impl ModelBackend for RemoteModelClient {
    fn predict(&self, context: &str, input: &str) -> Vec<PredictionResult> {
        // Synchronous wrapper - in real use, this should be async
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.predict_async(context, input))
    }
    
    fn is_available(&self) -> bool {
        true // Assume available, actual check would be async
    }
}

/// Chat completion request structure
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Chat completion response structure
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

/// Local model client (using candle or similar)
#[cfg(feature = "local-model")]
pub struct LocalModelClient {
    config: ModelConfig,
    // Model and tokenizer would be loaded here
    // tokenizer: tokenizers::Tokenizer,
    // model: Box<dyn candle_core::Module>,
}

#[cfg(feature = "local-model")]
impl LocalModelClient {
    pub fn new(config: ModelConfig) -> Result<Self, String> {
        // Load model and tokenizer
        // This is a placeholder for actual model loading
        Ok(Self { config })
    }
}

#[cfg(feature = "local-model")]
impl ModelBackend for LocalModelClient {
    fn predict(&self, _context: &str, _input: &str) -> Vec<PredictionResult> {
        // Implement local model inference
        vec![]
    }
    
    fn is_available(&self) -> bool {
        true
    }
}

/// Hybrid model client that combines multiple sources
pub struct HybridModelClient {
    remote: Option<RemoteModelClient>,
    #[cfg(feature = "local-model")]
    local: Option<LocalModelClient>,
}

impl HybridModelClient {
    pub fn new(config: ModelConfig) -> Self {
        let remote = match config.model_type.as_str() {
            "remote" | "hybrid" => Some(RemoteModelClient::new(config.clone())),
            _ => None,
        };
        
        #[cfg(feature = "local-model")]
        let local = match config.model_type.as_str() {
            "local" | "hybrid" => LocalModelClient::new(config.clone()).ok(),
            _ => None,
        };
        
        #[cfg(not(feature = "local-model"))]
        let _ = config;
        
        Self {
            remote,
            #[cfg(feature = "local-model")]
            local,
        }
    }
}

impl ModelBackend for HybridModelClient {
    fn predict(&self, context: &str, input: &str) -> Vec<PredictionResult> {
        let mut results = Vec::new();
        
        // Try remote model first
        if let Some(ref remote) = self.remote {
            if remote.is_available() {
                let remote_results = remote.predict(context, input);
                results.extend(remote_results);
            }
        }
        
        // Try local model
        #[cfg(feature = "local-model")]
        if let Some(ref local) = self.local {
            if local.is_available() && results.is_empty() {
                let local_results = local.predict(context, input);
                results.extend(local_results);
            }
        }
        
        results
    }
    
    fn is_available(&self) -> bool {
        let remote_available = self.remote.as_ref().map(|r| r.is_available()).unwrap_or(false);
        
        #[cfg(feature = "local-model")]
        {
            remote_available || self.local.as_ref().map(|l| l.is_available()).unwrap_or(false)
        }
        
        #[cfg(not(feature = "local-model"))]
        {
            remote_available
        }
    }
}

/// Model client factory
pub fn create_model_client(config: ModelConfig) -> Box<dyn ModelBackend> {
    match config.model_type.as_str() {
        "remote" => Box::new(RemoteModelClient::new(config)),
        #[cfg(feature = "local-model")]
        "local" => Box::new(LocalModelClient::new(config).unwrap()),
        "none" | "disabled" => Box::new(NoOpModelClient),
        _ => Box::new(HybridModelClient::new(config)),
    }
}

/// No-op model client for when AI is disabled
struct NoOpModelClient;

impl ModelBackend for NoOpModelClient {
    fn predict(&self, _context: &str, _input: &str) -> Vec<PredictionResult> {
        Vec::new()
    }

    fn is_available(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_remote_client_creation() {
        let config = ModelConfig::default();
        let client = RemoteModelClient::new(config);
        assert!(client.is_available());
    }
    
    #[test]
    fn test_prompt_building() {
        let config = ModelConfig::default();
        let client = RemoteModelClient::new(config);
        let prompt = client.build_prompt("你好", "nihao");
        assert!(prompt.contains("你好"));
        assert!(prompt.contains("nihao"));
    }
}
