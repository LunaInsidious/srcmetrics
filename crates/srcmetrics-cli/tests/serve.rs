//! HTTP API / UI end-to-end tests (ADR-0021). Starts `srcmetrics serve` on a free port.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};

struct Server {
    child: Child,
    address: String,
}

impl Server {
    fn start() -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_srcmetrics"))
            .args(["serve", "--port", "0"])
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut line = String::new();
        BufReader::new(child.stderr.as_mut().unwrap())
            .read_line(&mut line)
            .unwrap();
        let address = line
            .trim()
            .strip_prefix("listening on http://")
            .expect(&line)
            .to_string();
        Server { child, address }
    }

    /// Sends one HTTP/1.1 request and returns (status, body).
    fn request(&self, method: &str, path: &str, body: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(&self.address).unwrap();
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            self.address,
            body.len()
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let status = response.split(' ').nth(1).unwrap().parse().unwrap();
        let body = response.split_once("\r\n\r\n").unwrap().1.to_string();
        (status, body)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}

#[test]
fn serves_ui_and_analysis() {
    let server = Server::start();

    let (status, page) = server.request("GET", "/", "");
    assert_eq!(status, 200);
    assert!(page.contains("<textarea"));

    // Metric definitions are documentation, not an API (ADR-0026).
    assert_eq!(server.request("GET", "/api/metrics", "").0, 404);

    let request = r#"{"filename": "a.py", "source": "def f(x):\n    return x\n"}"#;
    let (status, body) = server.request("POST", "/api/analyze", request);
    assert_eq!(status, 200, "{body}");
    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(value["files"][0]["functions"][0]["name"], "f");

    let (status, body) = server.request(
        "POST",
        "/api/analyze",
        r#"{"filename": "a.txt", "source": ""}"#,
    );
    assert_eq!(status, 400);
    assert!(body.contains("unsupported file extension"), "{body}");

    let (status, body) = server.request(
        "POST",
        "/api/analyze",
        r#"{"filename": "a.py", "source": "def (:\n"}"#,
    );
    assert_eq!(status, 400);
    assert!(body.contains("parse error"), "{body}");
}
