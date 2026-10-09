//! `LocalNet` answers `data:` and absolute `file:` requests on the runtime's blocking pool, and
//! nothing else.

use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use blitz_kit::net::LocalNet;
use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request, Url};

struct Reply(Sender<(String, Vec<u8>)>);

impl NetHandler for Reply {
    fn bytes(self: Box<Self>, resolved_url: String, bytes: Bytes) {
        let _ = self.0.send((resolved_url, bytes.to_vec()));
    }
}

fn fetch(url: &str) -> Option<(String, Vec<u8>)> {
    let (tx, rx) = channel();
    let url: Url = url.parse().expect("a url");
    LocalNet.fetch(0, Request::get(url), Box::new(Reply(tx)));
    rx.recv_timeout(Duration::from_millis(500)).ok()
}

#[tokio::test]
async fn a_data_url_is_decoded() {
    let (url, bytes) = fetch("data:text/plain;base64,aGVsbG8=").expect("answered");
    assert_eq!(url, "data:text/plain;base64,aGVsbG8=");
    assert_eq!(bytes, b"hello");
}

#[tokio::test]
async fn an_absolute_file_url_is_read() {
    let path = std::env::temp_dir().join(format!("blitz-kit-net-{}.txt", std::process::id()));
    std::fs::write(&path, b"from disk").expect("scratch file");
    let url = Url::from_file_path(&path).expect("absolute").to_string();
    let answered = fetch(&url);
    let _ = std::fs::remove_file(&path);
    assert_eq!(answered.expect("answered").1, b"from disk");
}

#[tokio::test]
async fn anything_else_is_left_unanswered() {
    assert!(fetch("https://example.com/a.png").is_none());
}
