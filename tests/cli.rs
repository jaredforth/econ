use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use thrift::{parse_tier, pick_tier, run};

/// One-shot fake Ollama: reads the request, answers with `body`, returns host:port.
fn fake(body: &'static str) -> String {
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
        let _ = write!(
            s,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
    });
    addr.to_string()
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
    assert!(run(&a(&["a", "b"]), |_| None).is_err());
    let host = fake(HARD); // no scheme: run() adds http://
    let env = |k: &str| match k {
        "OLLAMA_HOST" => Some(host.clone()),
        "THRIFT_ROUTER_MODEL" => Some("tiny".into()),
        "THRIFT_HARD" => Some("opus".into()),
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
    let bin = env!("CARGO_BIN_EXE_thrift");
    let out = Command::new(bin).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage"));
    let out = Command::new(bin)
        .arg("hi")
        .env("OLLAMA_HOST", fake(SIMPLE))
        .env("THRIFT_SIMPLE", "haiku")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "haiku");
}
