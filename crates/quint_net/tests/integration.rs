use quint_net::{FetchError, fetch};

#[test]
#[ignore]
fn fetch_example_dot_com() {
    let resp = fetch("https://example.com").unwrap();
    assert_eq!(resp.status, 200);
    assert!(resp.body.to_lowercase().contains("<html"));
}

#[test]
#[ignore]
fn fetch_follows_redirects() {
    let resp = fetch("https://httpbin.org/redirect/1").unwrap();
    assert_eq!(resp.status, 200);
}

#[test]
#[ignore]
fn fetch_nonexistent_domain() {
    let err = fetch("https://this-domain-does-not-exist-quint-test.example").unwrap_err();
    assert!(matches!(err, FetchError::Network(_)));
}
