use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use url::Url;
use zeroize::Zeroizing;

const CALLBACK_ADDRESS: &str = "127.0.0.1:43817";
const CALLBACK_PATH: &str = "/auth/callback";
const MAX_REQUEST_BYTES: usize = 8 * 1024;

pub struct OAuthCallback {
    listener: TcpListener,
    local_addr: SocketAddr,
    redirect_url: Url,
    callback_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthCallbackError {
    #[error("the local sign-in callback is already in use")]
    Bind,
    #[error("the local sign-in callback must use loopback")]
    NotLoopback,
    #[error("the sign-in callback timed out")]
    Timeout,
    #[error("the sign-in callback request is invalid")]
    InvalidRequest,
    #[error("the sign-in callback request was too large")]
    RequestTooLarge,
    #[error("the sign-in callback state did not match")]
    StateMismatch,
    #[error("Google sign-in was cancelled or rejected")]
    ProviderRejected,
    #[error("the local sign-in callback failed")]
    Io,
}

impl OAuthCallback {
    pub async fn bind() -> Result<Self, OAuthCallbackError> {
        Self::bind_at(CALLBACK_ADDRESS, CALLBACK_PATH).await
    }

    pub async fn bind_to(address: &str) -> Result<Self, OAuthCallbackError> {
        Self::bind_at(address, CALLBACK_PATH).await
    }

    pub async fn bind_google_backup() -> Result<Self, OAuthCallbackError> {
        Self::bind_at("127.0.0.1:43818", "/backup/google").await
    }

    pub async fn bind_onedrive_backup() -> Result<Self, OAuthCallbackError> {
        Self::bind_at("127.0.0.1:43819", "/backup/onedrive").await
    }

    pub async fn bind_at(address: &str, callback_path: &str) -> Result<Self, OAuthCallbackError> {
        let requested: SocketAddr = address
            .parse()
            .map_err(|_| OAuthCallbackError::NotLoopback)?;
        if !requested.ip().is_loopback()
            || !callback_path.starts_with('/')
            || callback_path.contains('?')
            || callback_path.contains('#')
        {
            return Err(OAuthCallbackError::NotLoopback);
        }
        let listener = TcpListener::bind(requested)
            .await
            .map_err(|_| OAuthCallbackError::Bind)?;
        let local_addr = listener.local_addr().map_err(|_| OAuthCallbackError::Io)?;
        let redirect_url = Url::parse(&format!("http://{local_addr}{callback_path}"))
            .map_err(|_| OAuthCallbackError::Io)?;
        Ok(Self {
            listener,
            local_addr,
            redirect_url,
            callback_path: callback_path.to_owned(),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn redirect_url(&self) -> &Url {
        &self.redirect_url
    }

    pub async fn wait(
        self,
        expected_state: &str,
        wait_for: Duration,
    ) -> Result<Zeroizing<String>, OAuthCallbackError> {
        match timeout(wait_for, self.receive(expected_state)).await {
            Ok(result) => result,
            Err(_) => Err(OAuthCallbackError::Timeout),
        }
    }

    async fn receive(self, expected_state: &str) -> Result<Zeroizing<String>, OAuthCallbackError> {
        let (mut stream, peer) = self
            .listener
            .accept()
            .await
            .map_err(|_| OAuthCallbackError::Io)?;
        if !peer.ip().is_loopback() {
            write_error(&mut stream).await;
            return Err(OAuthCallbackError::NotLoopback);
        }
        let result = read_callback(&mut stream, expected_state, &self.callback_path).await;
        if result.is_ok() {
            write_success(&mut stream).await;
        } else {
            write_error(&mut stream).await;
        }
        result
    }
}

async fn read_callback(
    stream: &mut TcpStream,
    expected_state: &str,
    callback_path: &str,
) -> Result<Zeroizing<String>, OAuthCallbackError> {
    let mut request = Zeroizing::new(Vec::with_capacity(1024));
    loop {
        let remaining = MAX_REQUEST_BYTES + 1 - request.len();
        let mut buffer = [0_u8; 1024];
        let read_len = remaining.min(buffer.len());
        let read = stream
            .read(&mut buffer[..read_len])
            .await
            .map_err(|_| OAuthCallbackError::Io)?;
        if read == 0 {
            return Err(OAuthCallbackError::InvalidRequest);
        }
        request.extend_from_slice(&buffer[..read]);
        if request.len() > MAX_REQUEST_BYTES {
            return Err(OAuthCallbackError::RequestTooLarge);
        }
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    let request = std::str::from_utf8(&request).map_err(|_| OAuthCallbackError::InvalidRequest)?;
    let first_line = request
        .split("\r\n")
        .next()
        .ok_or(OAuthCallbackError::InvalidRequest)?;
    let mut parts = first_line.split_whitespace();
    let method = parts.next().ok_or(OAuthCallbackError::InvalidRequest)?;
    let target = parts.next().ok_or(OAuthCallbackError::InvalidRequest)?;
    let version = parts.next().ok_or(OAuthCallbackError::InvalidRequest)?;
    if method != "GET" || !version.starts_with("HTTP/1.") || parts.next().is_some() {
        return Err(OAuthCallbackError::InvalidRequest);
    }

    let parsed = Url::parse(&format!("http://127.0.0.1{target}"))
        .map_err(|_| OAuthCallbackError::InvalidRequest)?;
    if parsed.path() != callback_path {
        return Err(OAuthCallbackError::InvalidRequest);
    }
    let mut code = None;
    let mut state = None;
    let mut provider_error = false;
    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "code" if code.is_none() && !value.is_empty() => code = Some(value.into_owned()),
            "state" | "nn_state" if state.is_none() && !value.is_empty() => {
                state = Some(value.into_owned())
            }
            "error" => provider_error = true,
            "code" | "state" | "nn_state" => return Err(OAuthCallbackError::InvalidRequest),
            _ => {}
        }
    }
    if provider_error {
        return Err(OAuthCallbackError::ProviderRejected);
    }
    if state.as_deref() != Some(expected_state) {
        return Err(OAuthCallbackError::StateMismatch);
    }
    code.map(Zeroizing::new)
        .ok_or(OAuthCallbackError::InvalidRequest)
}

async fn write_success(stream: &mut TcpStream) {
    const BODY: &str = "<!doctype html><meta charset=utf-8><title>Noor Notes</title><p>Sign-in complete. You can return to Noor Notes.</p>";
    write_response(stream, "200 OK", BODY).await;
}

async fn write_error(stream: &mut TcpStream) {
    const BODY: &str = "<!doctype html><meta charset=utf-8><title>Noor Notes</title><p>Sign-in could not be completed. Return to Noor Notes and try again.</p>";
    write_response(stream, "400 Bad Request", BODY).await;
}

async fn write_response(stream: &mut TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
}
