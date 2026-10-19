use serde::{Deserialize, Serialize};

// What we send to Ollama
#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

// What Ollama sends back (we only keep the "response" field)
#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder().no_proxy().build()?;

    let body = GenerateRequest {
        model: "llama3.2:3b",
        prompt: "Say hi in five words",
        stream: false,
    };

    let reply: GenerateResponse = client
        .post("http://localhost:11434/api/generate")
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    println!("Model said: {}", reply.response);
    Ok(())
}
