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

// ---------- Ollama's own API ----------
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

pub struct OllamaTarget {
    client: reqwest::Client,
    base_url: String,
    model: String,
    system: String,
}

impl OllamaTarget {
    pub fn new(base_url: &str, model: &str, system: &str) -> Result<Self, BoxError> {
        let client = reqwest::Client::builder().no_proxy().build()?;
        Ok(OllamaTarget {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
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
            .post(format!("{}/api/generate", self.base_url))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(reply.response)
    }
}

// ---------- OpenAI-style API ----------
// Works with OpenAI, many hosted models, and Ollama's /v1 address.
#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatReply,
}

#[derive(Deserialize)]
struct ChatReply {
    content: Option<String>,
}

pub struct OpenAiTarget {
    client: reqwest::Client,
    base_url: String,
    model: String,
    system: String,
    api_key: Option<String>,
}

impl OpenAiTarget {
    pub fn new(
        base_url: &str,
        model: &str,
        system: &str,
        api_key: Option<String>,
    ) -> Result<Self, BoxError> {
        let client = reqwest::Client::builder().no_proxy().build()?;
        Ok(OpenAiTarget {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            system: system.to_string(),
            api_key,
        })
    }
}

#[async_trait]
impl Target for OpenAiTarget {
    fn name(&self) -> String {
        format!("openai-style:{}", self.model)
    }

    async fn ask(&self, prompt: &str) -> Result<String, BoxError> {
        let body = ChatRequest {
            model: &self.model,
            messages: vec![
                ChatMessage { role: "system", content: &self.system },
                ChatMessage { role: "user", content: prompt },
            ],
            stream: false,
        };
        let mut request = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&body);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        let response = request.send().await?.error_for_status()?;
        let parsed: ChatResponse = response.json().await?;
        let text = parsed
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();
        Ok(text)
    }
}

// ---------- A fake model for demos and tests ----------
// Leaks only when asked in JSON framing.
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

    #[test]
    fn parses_openai_style_reply() {
        let raw = r#"{"choices":[{"message":{"role":"assistant","content":"hi"}}]}"#;
        let parsed: ChatResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.choices[0].message.content.as_deref(), Some("hi"));
    }

    #[test]
    fn openai_style_reply_with_null_content_is_ok() {
        let raw = r#"{"choices":[{"message":{"content":null}}]}"#;
        let parsed: ChatResponse = serde_json::from_str(raw).unwrap();
        assert!(parsed.choices[0].message.content.is_none());
    }
}
