use econ::{config_path, parse_claude, parse_config, parse_tier, pick_tier, pick_tier_claude, run};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver};

/// One-shot fake Ollama: reads the request, answers with `body`, returns host:port.
fn fake(body: &'static str) -> String {
    fake_capture(body).0
}

/// Like `fake`, but also hands back the raw request text.
fn fake_capture(body: &'static str) -> (String, Receiver<String>) {
    let (tx, rx) = channel();
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut s, _) = l.accept().unwrap();
        let mut buf = [0u8; 65536];
        let mut got = Vec::new();
        while !got.ends_with(b"}") {
            let n = s.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            got.extend_from_slice(&buf[..n]);
        }
        let _ = tx.send(String::from_utf8_lossy(&got).into_owned());
        let _ = write!(
            s,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
    });
    (addr.to_string(), rx)
}

const HARD: &str = r#"{"message":{"content":"{\"tier\":\"hard\"}"}}"#;
const SIMPLE: &str = r#"{"message":{"content":"{\"tier\":\"simple\"}"}}"#;

fn j(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap()
}

fn a(s: &[&str]) -> Vec<String> {
    s.iter().map(|x| x.to_string()).collect()
}

#[test]
fn parses() {
    assert_eq!(parse_tier(&j(HARD)), Some("hard"));
    assert_eq!(parse_tier(&j("{\"a\":1}")), None);
    assert_eq!(parse_tier(&j("{}")), None);
    assert_eq!(parse_tier(&j(r#"{"message":{"content":"x"}}"#)), None);
    assert_eq!(
        parse_tier(&j(r#"{"message":{"content":"{\"tier\":\"huge\"}"}}"#)),
        None
    );
}

#[test]
fn picks_from_server_and_truncates() {
    let h = format!("http://{}/", fake(SIMPLE));
    assert_eq!(pick_tier(&h, "m", &"é".repeat(5000)), "simple");
}

#[test]
fn falls_back_to_moderate() {
    assert_eq!(pick_tier("http://127.0.0.1:1", "m", "x"), "moderate");
    let h = format!("http://{}", fake("garbage"));
    assert_eq!(pick_tier(&h, "m", "x"), "moderate");
}

#[test]
fn run_args_and_env() {
    assert!(run(&a(&[]), |_| None).is_err());
    let host = fake(HARD); // no scheme: run() adds http://
    let env = |k: &str| match k {
        "OLLAMA_HOST" => Some(host.clone()),
        "ECON_ROUTER_MODEL" => Some("tiny".into()),
        "ECON_HARD" => Some("opus".into()),
        _ => None,
    };
    assert_eq!(run(&a(&["design a db"]), env).unwrap(), "opus");
    // tier model unset -> tier name; router unreachable -> moderate
    let down = |k: &str| (k == "OLLAMA_HOST").then(|| "http://127.0.0.1:1".to_string());
    assert_eq!(run(&a(&["x"]), down).unwrap(), "moderate");
    // all defaults
    assert!(run(&a(&["x"]), |_| None).is_ok());
}

#[test]
fn binary() {
    let bin = env!("CARGO_BIN_EXE_econ");
    let out = Command::new(bin)
        .env("ECON_CONFIG", "/nonexistent")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no prompt"));
    let out = Command::new(bin)
        .env("ECON_CONFIG", "/nonexistent")
        .arg("hi")
        .env("OLLAMA_HOST", fake(SIMPLE))
        .env("ECON_SIMPLE", "haiku")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "haiku");
}

const CLAUDE_HARD: &str = r#"{"content":[{"type":"text","text":"Hard"}]}"#;

#[test]
fn parses_claude() {
    assert_eq!(parse_claude(&j(CLAUDE_HARD)), Some("hard"));
    assert_eq!(parse_claude(&j("{}")), None);
    assert_eq!(parse_claude(&j(r#"{"content":[{"text":"dunno"}]}"#)), None);
}

#[test]
fn claude_request_and_fallback() {
    let (host, rx) = fake_capture(CLAUDE_HARD);
    assert_eq!(
        pick_tier_claude(&format!("http://{host}/"), "k", "claude-x", "hi"),
        "hard"
    );
    let req = rx.recv().unwrap().to_lowercase();
    assert!(req.contains("post /v1/messages"));
    assert!(req.contains("x-api-key: k"));
    assert!(req.contains("anthropic-version: 2023-06-01"));
    assert_eq!(
        pick_tier_claude("http://127.0.0.1:1", "k", "m", "x"),
        "moderate"
    );
}

#[test]
fn run_with_haiku() {
    let (host, rx) = fake_capture(CLAUDE_HARD);
    let env = |k: &str| match k {
        "ECON_ROUTER_MODEL" => Some("haiku".into()),
        "ANTHROPIC_API_KEY" => Some("k".into()),
        "ANTHROPIC_BASE_URL" => Some(format!("http://{host}")),
        "ECON_HARD" => Some("opus".into()),
        _ => None,
    };
    assert_eq!(run(&a(&["design a db"]), env).unwrap(), "opus");
    assert!(rx.recv().unwrap().contains("claude-haiku-5-5"));
    // full model id passes through; no base url set -> real API default is used (unreachable key -> moderate or tier)
    let (host, rx) = fake_capture(CLAUDE_HARD);
    let env = |k: &str| match k {
        "ECON_ROUTER_MODEL" => Some("claude-sonnet-5-5".into()),
        "ANTHROPIC_API_KEY" => Some("k".into()),
        "ANTHROPIC_BASE_URL" => Some(format!("http://{host}")),
        _ => None,
    };
    assert_eq!(run(&a(&["x"]), env).unwrap(), "hard");
    assert!(rx.recv().unwrap().contains("claude-sonnet-5-5"));
    // missing key is an error
    let env = |k: &str| (k == "ECON_ROUTER_MODEL").then(|| "haiku".to_string());
    assert!(run(&a(&["x"]), env)
        .unwrap_err()
        .contains("ANTHROPIC_API_KEY"));
}

#[test]
fn config_file_parsing_and_path() {
    let c = parse_config(r#"{"ECON_ROUTER_MODEL": "haiku", "ECON_HARD": "opus"}"#).unwrap();
    assert_eq!(c["ECON_ROUTER_MODEL"], "haiku");
    assert_eq!(c.len(), 2);
    assert!(parse_config("not json")
        .unwrap_err()
        .contains("JSON object"));
    assert!(parse_config(r#"{"A": 1}"#).is_err());
    let get = |pairs: &'static [(&str, &str)]| {
        move |k: &str| pairs.iter().find(|p| p.0 == k).map(|p| p.1.to_string())
    };
    let p = |x: Option<std::path::PathBuf>| x.map(|p| p.to_string_lossy().into_owned());
    assert_eq!(p(config_path(get(&[("ECON_CONFIG", "/a")]))).unwrap(), "/a");
    assert_eq!(
        p(config_path(get(&[("XDG_CONFIG_HOME", "/x")]))).unwrap(),
        "/x/econ/config.json"
    );
    assert_eq!(
        p(config_path(get(&[("HOME", "/h")]))).unwrap(),
        "/h/.config/econ/config.json"
    );
    assert!(config_path(get(&[])).is_none());
}

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("econ-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("config.json")
}

#[test]
fn binary_reads_json_config_and_env_overrides_it() {
    let bin = env!("CARGO_BIN_EXE_econ");
    let cfg = tmp("bin");
    let host = fake(SIMPLE);
    std::fs::write(
        &cfg,
        format!(r#"{{"OLLAMA_HOST": "{host}", "ECON_SIMPLE": "from-file"}}"#),
    )
    .unwrap();
    let out = |c: &mut Command| {
        let o = c.arg("hi").env("ECON_CONFIG", &cfg).output().unwrap();
        (
            o.status.code(),
            String::from_utf8_lossy(&o.stdout).trim().to_string(),
            String::from_utf8_lossy(&o.stderr).to_string(),
        )
    };
    assert_eq!(out(&mut Command::new(bin)).1, "from-file");
    // env beats file (router unreachable -> tier name)
    let host2 = fake(SIMPLE);
    let o = out(Command::new(bin)
        .env("ECON_SIMPLE", "from-env")
        .env("OLLAMA_HOST", host2));
    assert_eq!(o.1, "from-env");
    // broken config is a clear error, exit 2
    std::fs::write(&cfg, "{oops").unwrap();
    let o = out(&mut Command::new(bin));
    assert_eq!(o.0, Some(2));
    assert!(o.2.contains("config.json") && o.2.contains("JSON object"));
    // missing file is fine
    std::fs::remove_file(&cfg).unwrap();
    assert_eq!(
        out(Command::new(bin).env("OLLAMA_HOST", "127.0.0.1:1")).1,
        "moderate"
    );
}

#[test]
fn user_env_layers_file_under_process_env() {
    let cfg = tmp("env");
    std::fs::write(&cfg, r#"{"ECON_TEST_A": "file", "ECON_TEST_B": "file"}"#).unwrap();
    std::env::set_var("ECON_CONFIG", &cfg);
    std::env::set_var("ECON_TEST_B", "env");
    let env = econ::user_env().unwrap();
    assert_eq!(env("ECON_TEST_A").unwrap(), "file");
    assert_eq!(env("ECON_TEST_B").unwrap(), "env");
    assert_eq!(env("ECON_TEST_C"), None);
    std::fs::write(&cfg, "bad").unwrap();
    assert!(econ::user_env().is_err());
    std::fs::remove_file(&cfg).unwrap();
    assert!(econ::user_env().is_ok());
    std::env::remove_var("ECON_CONFIG");
}

#[test]
fn init_writes_and_merges_config() {
    let cfg = tmp("init");
    let _ = std::fs::remove_file(&cfg);
    let env = |k: &str| (k == "ECON_CONFIG").then(|| cfg.to_string_lossy().into_owned());
    // fresh: defaults
    assert!(run(&a(&["init"]), env).unwrap().contains("wrote"));
    let c = parse_config(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
    assert_eq!(c["ECON_ROUTER_MODEL"], "qwen3:4b");
    assert_eq!(c["ECON_HARD"], "opus");
    // router arg overrides; custom keys and tier choices are kept
    std::fs::write(&cfg, r#"{"ECON_SIMPLE": "mini", "ANTHROPIC_API_KEY": "k"}"#).unwrap();
    run(&a(&["init", "haiku"]), env).unwrap();
    let c = parse_config(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
    assert_eq!(c["ECON_ROUTER_MODEL"], "haiku");
    assert_eq!(c["ECON_SIMPLE"], "mini");
    assert_eq!(c["ANTHROPIC_API_KEY"], "k");
    assert_eq!(c["ECON_MODERATE"], "sonnet");
    // re-running without an arg keeps the router
    run(&a(&["init"]), env).unwrap();
    let c = parse_config(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
    assert_eq!(c["ECON_ROUTER_MODEL"], "haiku");
    // errors: broken file, unwritable location, no location
    std::fs::write(&cfg, "bad").unwrap();
    assert!(run(&a(&["init"]), env).unwrap_err().contains("config.json"));
    let under_file = cfg.join("x/config.json"); // parent is a file
    let bad = |k: &str| (k == "ECON_CONFIG").then(|| under_file.to_string_lossy().into_owned());
    assert!(run(&a(&["init"]), bad).is_err());
    assert!(run(&a(&["init"]), |_| None).unwrap_err().contains("HOME"));
    // more than `init [router]` is just a prompt (router down -> moderate)
    let down = |k: &str| (k == "OLLAMA_HOST").then(|| "http://127.0.0.1:1".to_string());
    assert_eq!(run(&a(&["init", "a", "b"]), down).unwrap(), "moderate");
}

#[test]
fn cli_surface() {
    let down = |k: &str| (k == "OLLAMA_HOST").then(|| "http://127.0.0.1:1".to_string());
    assert!(run(&a(&["-h"]), down).unwrap().contains("usage:"));
    assert!(run(&a(&["--help"]), down).unwrap().contains("--tier"));
    assert!(run(&a(&["-V"]), down).unwrap().starts_with("econ 0."));
    assert!(run(&a(&["--version"]), down).is_ok());
    assert!(run(&a(&["--bogus"]), down)
        .unwrap_err()
        .contains("unknown option --bogus"));
    assert!(run(&a(&[]), down).unwrap_err().contains("no prompt"));
    assert!(run(&a(&["  "]), down).unwrap_err().contains("no prompt"));
    // unquoted multi-word prompt, `--` escape, --tier ignores tier->model mapping
    let mapped = |k: &str| match k {
        "OLLAMA_HOST" => Some("http://127.0.0.1:1".to_string()),
        "ECON_MODERATE" => Some("sonnet".to_string()),
        _ => None,
    };
    assert_eq!(run(&a(&["fix", "the", "typo"]), mapped).unwrap(), "sonnet");
    assert_eq!(run(&a(&["--", "-h", "me"]), mapped).unwrap(), "sonnet");
    assert_eq!(run(&a(&["--tier", "x"]), mapped).unwrap(), "moderate");
    assert_eq!(run(&a(&["x", "--tier"]), mapped).unwrap(), "moderate");
}

#[test]
fn binary_reads_stdin() {
    use std::process::Stdio;
    let mut c = Command::new(env!("CARGO_BIN_EXE_econ"))
        .env("ECON_CONFIG", "/nonexistent")
        .env("OLLAMA_HOST", fake(SIMPLE))
        .env("ECON_SIMPLE", "haiku")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin
        .take()
        .unwrap()
        .write_all(b"summarize this")
        .unwrap();
    let out = c.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "haiku");
}

#[test]
fn binary_init() {
    let cfg = tmp("binit");
    let _ = std::fs::remove_file(&cfg);
    let o = Command::new(env!("CARGO_BIN_EXE_econ"))
        .args(["init", "haiku"])
        .env("ECON_CONFIG", &cfg)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&o.stdout).contains("wrote"));
    assert!(std::fs::read_to_string(&cfg).unwrap().contains("\"haiku\""));
}
