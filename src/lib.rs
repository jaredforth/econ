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

fn post(url: &str, key: Option<&str>, body: Value) -> Option<Value> {
    let mut r = ureq::post(url).timeout(std::time::Duration::from_secs(30));
    if let Some(k) = key {
        r = r.set("x-api-key", k).set("anthropic-version", "2023-06-01");
    }
    r.send_json(body).ok()?.into_json().ok()
}

/// Tier named in an Anthropic /v1/messages response, or None if garbled.
pub fn parse_claude(v: &Value) -> Option<&'static str> {
    let t = v["content"][0]["text"].as_str()?.to_lowercase();
    TIERS.into_iter().find(|x| t.contains(x))
}

fn clip(prompt: &str) -> String {
    prompt.chars().take(4000).collect()
}

/// Ask a local Ollama model. Any failure falls back to "moderate": don't gamble on cheap.
pub fn pick_tier(host: &str, model: &str, prompt: &str) -> &'static str {
    let body = json!({
        "model": model, "stream": false, "think": false,
        "options": {"temperature": 0, "num_predict": 20},
        "format": {"type": "object", "properties": {"tier": {"enum": TIERS}}, "required": ["tier"]},
        "messages": [{"role": "system", "content": SYSTEM}, {"role": "user", "content": clip(prompt)}],
    });
    post(
        &format!("{}/api/chat", host.trim_end_matches('/')),
        None,
        body,
    )
    .and_then(|v| parse_tier(&v))
    .unwrap_or("moderate")
}

/// Ask a Claude model (e.g. Haiku) over the Anthropic API. Same fallback.
pub fn pick_tier_claude(base: &str, key: &str, model: &str, prompt: &str) -> &'static str {
    let body = json!({
        "model": model, "max_tokens": 20, "temperature": 0, "system": SYSTEM,
        "messages": [{"role": "user", "content": clip(prompt)}],
    });
    post(
        &format!("{}/v1/messages", base.trim_end_matches('/')),
        Some(key),
        body,
    )
    .and_then(|v| parse_claude(&v))
    .unwrap_or("moderate")
}

/// Config from env: THRIFT_ROUTER_MODEL (an Ollama model, `haiku`, or a `claude-*` id),
/// OLLAMA_HOST, ANTHROPIC_API_KEY, ANTHROPIC_BASE_URL, THRIFT_SIMPLE/MODERATE/HARD.
/// Prints the tier's model if THRIFT_<TIER> is set, else the tier name.
pub fn run(args: &[String], env: impl Fn(&str) -> Option<String>) -> Result<String, String> {
    run_dyn(args, &env)
}

fn run_dyn(args: &[String], env: &dyn Fn(&str) -> Option<String>) -> Result<String, String> {
    if args.first().map(String::as_str) == Some("init") && args.len() <= 2 {
        let path = config_path_dyn(env).ok_or("no config location: set HOME or THRIFT_CONFIG")?;
        return init(&path, args.get(1).map(String::as_str));
    }
    let [prompt] = args else {
        return Err("usage: thrift \"the prompt\" | thrift init [router-model]".into());
    };
    let model = env("THRIFT_ROUTER_MODEL").unwrap_or_else(|| "qwen3:4b".into());
    let tier = if model == "haiku" || model.starts_with("claude") {
        let key = env("ANTHROPIC_API_KEY").ok_or("set ANTHROPIC_API_KEY to route with Claude")?;
        let base = env("ANTHROPIC_BASE_URL").unwrap_or("https://api.anthropic.com".into());
        let model = if model == "haiku" {
            "claude-haiku-5-5"
        } else {
            &model
        };
        pick_tier_claude(&base, &key, model, prompt)
    } else {
        let mut host = env("OLLAMA_HOST").unwrap_or_else(|| "http://localhost:11434".into());
        if !host.contains("://") {
            host = format!("http://{host}");
        }
        pick_tier(&host, &model, prompt)
    };
    Ok(env(&format!("THRIFT_{}", tier.to_uppercase())).unwrap_or_else(|| tier.into()))
}

/// Flat JSON object of string values, e.g. {"THRIFT_ROUTER_MODEL": "haiku"}.
pub fn parse_config(text: &str) -> Result<std::collections::HashMap<String, String>, String> {
    serde_json::from_str(text).map_err(|e| format!("config must be a JSON object of strings: {e}"))
}

/// $THRIFT_CONFIG, else $XDG_CONFIG_HOME/thrift/config.json, else ~/.config/thrift/config.json.
pub fn config_path(env: impl Fn(&str) -> Option<String>) -> Option<std::path::PathBuf> {
    config_path_dyn(&env)
}

fn config_path_dyn(env: &dyn Fn(&str) -> Option<String>) -> Option<std::path::PathBuf> {
    if let Some(p) = env("THRIFT_CONFIG") {
        return Some(p.into());
    }
    let base = env("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| env("HOME").map(|h| std::path::Path::new(&h).join(".config")))?;
    Some(base.join("thrift/config.json"))
}

/// Process env first, then the config file (same key names). A missing file is fine; a broken one is an error.
pub fn user_env() -> Result<impl Fn(&str) -> Option<String>, String> {
    let file = match config_path(|k| std::env::var(k).ok())
        .map(|p| (std::fs::read_to_string(&p), p))
    {
        Some((Ok(text), p)) => parse_config(&text).map_err(|e| format!("{}: {e}", p.display()))?,
        _ => Default::default(),
    };
    Ok(move |k: &str| std::env::var(k).ok().or_else(|| file.get(k).cloned()))
}

/// Write/update the config file: set the router (default `qwen3:4b`, keeps the current one if
/// already set and none given) and fill in any missing tier models. Other keys are kept.
pub fn init(path: &std::path::Path, router: Option<&str>) -> Result<String, String> {
    let mut cfg = match std::fs::read_to_string(path) {
        Ok(text) => parse_config(&text).map_err(|e| format!("{}: {e}", path.display()))?,
        Err(_) => Default::default(),
    };
    let mut cfg: std::collections::BTreeMap<_, _> = cfg.drain().collect();
    if let Some(r) = router {
        cfg.insert("THRIFT_ROUTER_MODEL".into(), r.into());
    }
    for (k, v) in [
        ("THRIFT_ROUTER_MODEL", "qwen3:4b"),
        ("THRIFT_SIMPLE", "haiku"),
        ("THRIFT_MODERATE", "sonnet"),
        ("THRIFT_HARD", "opus"),
    ] {
        cfg.entry(k.into()).or_insert(v.into());
    }
    let text = serde_json::to_string_pretty(&cfg).unwrap() + "\n";
    let fail = |e: std::io::Error| format!("{}: {e}", path.display());
    std::fs::create_dir_all(path.parent().unwrap()).map_err(fail)?;
    std::fs::write(path, text).map_err(fail)?;
    Ok(format!("wrote {}", path.display()))
}
