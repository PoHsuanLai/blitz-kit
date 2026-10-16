//! `LocalNet` answers `data:` and absolute `file:` requests inside its roots on the runtime's
//! blocking pool, and nothing else.

use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use blitz_kit::net::{FileRoots, LocalNet};
use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request, Url};

struct Reply(Sender<(String, Vec<u8>)>);

impl NetHandler for Reply {
    fn bytes(self: Box<Self>, resolved_url: String, bytes: Bytes) {
        let _ = self.0.send((resolved_url, bytes.to_vec()));
    }
}

fn fetch_from(net: &LocalNet, url: &str) -> Option<(String, Vec<u8>)> {
    let (tx, rx) = channel();
    let url: Url = url.parse().expect("a url");
    net.fetch(0, Request::get(url), Box::new(Reply(tx)));
    rx.recv_timeout(Duration::from_millis(500)).ok()
}

fn fetch(url: &str) -> Option<(String, Vec<u8>)> {
    fetch_from(&LocalNet::default(), url)
}

#[tokio::test]
async fn a_data_url_is_decoded() {
    let (url, bytes) = fetch("data:text/plain;base64,aGVsbG8=").expect("answered");
    assert_eq!(url, "data:text/plain;base64,aGVsbG8=");
    assert_eq!(bytes, b"hello");
}

#[tokio::test]
async fn an_absolute_file_url_inside_the_roots_is_read() {
    let path = std::env::temp_dir().join(format!("blitz-kit-net-{}.txt", std::process::id()));
    std::fs::write(&path, b"from disk").expect("scratch file");
    let url = Url::from_file_path(&path).expect("absolute").to_string();
    let net = LocalNet::new(FileRoots::new([std::env::temp_dir()]));
    let answered = fetch_from(&net, &url);
    let _ = std::fs::remove_file(&path);
    assert_eq!(answered.expect("answered").1, b"from disk");
}

#[tokio::test]
async fn a_file_url_the_roots_do_not_hold_is_left_unanswered() {
    let path = std::env::temp_dir().join(format!("blitz-kit-net-no-{}.txt", std::process::id()));
    std::fs::write(&path, b"private").expect("scratch file");
    let url = Url::from_file_path(&path).expect("absolute").to_string();
    let elsewhere = LocalNet::new(FileRoots::new([std::path::PathBuf::from("/nonexistent")]));
    let refused = fetch_from(&elsewhere, &url);
    let by_default = fetch(&url);
    let _ = std::fs::remove_file(&path);
    assert!(refused.is_none(), "outside the roots");
    assert!(by_default.is_none(), "no roots at all");
}

#[tokio::test]
async fn anything_else_is_left_unanswered() {
    assert!(fetch("https://example.com/a.png").is_none());
}
