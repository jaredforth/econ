//! `thrift "prompt"`: a small local model picks the cheapest tier that can handle the prompt.
use serde_json::{json, Value};

pub const TIERS: [&str; 3] = ["simple", "moderate", "hard"];
const SYSTEM: &str = "Classify how much reasoning the user's request needs. \
simple: lookup, rewrite, summarize, extract, short factual or formatting task. \
moderate: ordinary coding, analysis or writing with a few steps. \
hard: multi-step reasoning, architecture, hard debugging, math, high-stakes judgment. \
Answer with the tier only.";

/// Tier from an Ollama /api/chat response, or None if garbled.
pub fn parse_tier(v: &Value) -> Option<&'static str> {
    let inner: Value = serde_json::from_str(v["message"]["content"].as_str()?).ok()?;
    TIERS.into_iter().find(|t| inner["tier"] == *t)
}

/// Ask the router model. Any failure falls back to "moderate": don't gamble on cheap.
pub fn pick_tier(host: &str, model: &str, prompt: &str) -> &'static str {
    let body = json!({
        "model": model, "stream": false, "think": false,
        "options": {"temperature": 0, "num_predict": 20},
        "format": {"type": "object", "properties": {"tier": {"enum": TIERS}}, "required": ["tier"]},
        "messages": [
            {"role": "system", "content": SYSTEM},
            {"role": "user", "content": prompt.chars().take(4000).collect::<String>()},
        ],
    });
    let url = format!("{}/api/chat", host.trim_end_matches('/'));
    ureq::post(&url)
        .timeout(std::time::Duration::from_secs(30))
        .send_json(body)
        .ok()
        .and_then(|r| r.into_json::<Value>().ok())
        .and_then(|v| parse_tier(&v))
        .unwrap_or("moderate")
}

/// Config from env: THRIFT_ROUTER_MODEL, OLLAMA_HOST, THRIFT_SIMPLE/MODERATE/HARD.
/// Prints the tier's model if THRIFT_<TIER> is set, else the tier name.
pub fn run(args: &[String], env: impl Fn(&str) -> Option<String>) -> Result<String, String> {
    let [prompt] = args else {
        return Err("usage: thrift \"the prompt\"".into());
    };
    let model = env("THRIFT_ROUTER_MODEL").unwrap_or_else(|| "qwen3:4b".into());
    let mut host = env("OLLAMA_HOST").unwrap_or_else(|| "http://localhost:11434".into());
    if !host.contains("://") {
        host = format!("http://{host}");
    }
    let tier = pick_tier(&host, &model, prompt);
    Ok(env(&format!("THRIFT_{}", tier.to_uppercase())).unwrap_or_else(|| tier.into()))
}
