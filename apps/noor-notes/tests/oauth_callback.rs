use std::time::Duration;

use noor_notes::oauth_callback::{OAuthCallback, OAuthCallbackError};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

async fn request(address: std::net::SocketAddr, request: &[u8]) -> String {
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream.write_all(request).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    String::from_utf8(response).unwrap()
}

#[tokio::test]
async fn callback_accepts_one_matching_state_and_returns_a_success_page() {
    let callback = OAuthCallback::bind_to("127.0.0.1:0").await.unwrap();
    let address = callback.local_addr();
    let redirect = callback.redirect_url().clone();
    let waiter = tokio::spawn(callback.wait("expected-state", Duration::from_secs(2)));
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream
        .write_all(
            b"GET /auth/callback?code=one-time-code&nn_state=expected-state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        )
        .await
        .unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();

    let code = waiter.await.unwrap().unwrap();

    assert_eq!(code.as_str(), "one-time-code");
    assert_eq!(redirect.scheme(), "http");
    assert_eq!(redirect.host_str(), Some("127.0.0.1"));
    assert_eq!(redirect.path(), "/auth/callback");
    assert!(String::from_utf8(response).unwrap().contains("200 OK"));
}

#[tokio::test]
async fn callback_rejects_wrong_state_without_exposing_the_code() {
    let callback = OAuthCallback::bind_to("127.0.0.1:0").await.unwrap();
    let address = callback.local_addr();
    let waiter = tokio::spawn(callback.wait("expected-state", Duration::from_secs(2)));
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream
        .write_all(
            b"GET /auth/callback?code=secret-code&nn_state=wrong HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        )
        .await
        .unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();

    let error = match waiter.await.unwrap() {
        Err(error) => error,
        Ok(_) => panic!("wrong state must fail"),
    };

    assert!(matches!(error, OAuthCallbackError::StateMismatch));
    assert!(!error.to_string().contains("secret-code"));
    assert!(
        String::from_utf8(response)
            .unwrap()
            .contains("400 Bad Request")
    );
}

#[tokio::test]
async fn callback_rejects_non_get_and_oversized_requests() {
    let callback = OAuthCallback::bind_to("127.0.0.1:0").await.unwrap();
    let address = callback.local_addr();
    let waiter = tokio::spawn(callback.wait("expected-state", Duration::from_secs(2)));
    let response = request(
        address,
        b"POST /auth/callback HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
    )
    .await;
    let error = match waiter.await.unwrap() {
        Err(error) => error,
        Ok(_) => panic!("POST must fail"),
    };
    assert!(matches!(error, OAuthCallbackError::InvalidRequest));
    assert!(response.contains("400 Bad Request"));

    let callback = OAuthCallback::bind_to("127.0.0.1:0").await.unwrap();
    let address = callback.local_addr();
    let waiter = tokio::spawn(callback.wait("expected-state", Duration::from_secs(2)));
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream.write_all(&vec![b'A'; 8_193]).await.unwrap();
    stream.shutdown().await.unwrap();
    let error = match waiter.await.unwrap() {
        Err(error) => error,
        Ok(_) => panic!("oversized request must fail"),
    };
    assert!(matches!(error, OAuthCallbackError::RequestTooLarge));
}
