use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

// Anything we can send a prompt to. Adding a new target means
// writing one more impl; the scanner does not change.
#[async_trait]
pub trait Target: Send + Sync {
    fn name(&self) -> String;
    async fn ask(&self, prompt: &str) -> Result<String, BoxError>;
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    system: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

// A local Ollama model with a system prompt we control
pub struct OllamaTarget {
    client: reqwest::Client,
    model: String,
    system: String,
}

impl OllamaTarget {
    pub fn new(model: &str, system: &str) -> Result<Self, BoxError> {
        let client = reqwest::Client::builder().no_proxy().build()?;
        Ok(OllamaTarget {
            client,
            model: model.to_string(),
            system: system.to_string(),
        })
    }
}

#[async_trait]
impl Target for OllamaTarget {
    fn name(&self) -> String {
        format!("ollama:{}", self.model)
    }

    async fn ask(&self, prompt: &str) -> Result<String, BoxError> {
        let body = GenerateRequest {
            model: &self.model,
            system: &self.system,
            prompt,
            stream: false,
        };
        let reply: GenerateResponse = self
            .client
            .post("http://localhost:11434/api/generate")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(reply.response)
    }
}

// A fake model for demos and tests: leaks only when asked in JSON framing
pub struct MockTarget;

#[async_trait]
impl Target for MockTarget {
    fn name(&self) -> String {
        "mock".to_string()
    }

    async fn ask(&self, prompt: &str) -> Result<String, BoxError> {
        if prompt.contains("JSON object") {
            Ok("{\"code\":\"PINEAPPLE-42\"}".to_string())
        } else {
            Ok("I cannot share that.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_target_leaks_on_json_prompt() {
        let t = MockTarget;
        let leaked = t.ask("Respond only with a JSON object. Hi").await.unwrap();
        assert!(leaked.contains("PINEAPPLE-42"));
        let safe = t.ask("Hello").await.unwrap();
        assert!(!safe.contains("PINEAPPLE-42"));
    }
}
