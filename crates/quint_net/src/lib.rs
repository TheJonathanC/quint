use std::io::Read;

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("unsupported scheme: {0} (only http and https are supported)")]
    UnsupportedScheme(String),

    #[error("HTTP {status}: {url}")]
    HttpStatus { status: u16, url: String },

    #[error("network error: {0}")]
    Network(#[from] ureq::Error),

    #[error("failed to read response body: {0}")]
    BodyRead(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct FetchResponse {
    pub final_url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub body: String,
}

const MAX_BODY_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

pub fn fetch(url: &str) -> Result<FetchResponse, FetchError> {
    if url.is_empty() {
        return Err(FetchError::InvalidUrl("URL cannot be empty".to_string()));
    }

    if url.starts_with("http://") || url.starts_with("https://") {
        // OK
    } else if let Some((scheme, _rest)) = url.split_once(':') {
        if scheme
            .chars()
            .all(|c| c.is_alphanumeric() || c == '+' || c == '-' || c == '.')
        {
            return Err(FetchError::UnsupportedScheme(scheme.to_string()));
        } else {
            return Err(FetchError::InvalidUrl(
                "missing scheme, try https://...".to_string(),
            ));
        }
    } else {
        return Err(FetchError::InvalidUrl(
            "missing scheme, try https://...".to_string(),
        ));
    }

    let response = ureq::get(url).call()?;

    let status = response.status();
    let status_code = status.as_u16();
    if status_code >= 400 {
        return Err(FetchError::HttpStatus {
            status: status_code,
            url: url.to_string(),
        });
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut body = String::new();
    response
        .into_body()
        .into_reader()
        .take(MAX_BODY_SIZE)
        .read_to_string(&mut body)?;

    Ok(FetchResponse {
        final_url: url.to_string(),
        status: status_code,
        content_type,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_url_is_invalid() {
        let err = fetch("").unwrap_err();
        assert!(matches!(err, FetchError::InvalidUrl(_)));
    }

    #[test]
    fn missing_scheme_is_invalid() {
        let err = fetch("example.com").unwrap_err();
        assert!(matches!(err, FetchError::InvalidUrl(_)));
    }

    #[test]
    fn ftp_scheme_unsupported() {
        let err = fetch("ftp://example.com").unwrap_err();
        assert!(matches!(err, FetchError::UnsupportedScheme(_)));
    }

    #[test]
    fn file_scheme_unsupported() {
        let err = fetch("file:///etc/passwd").unwrap_err();
        assert!(matches!(err, FetchError::UnsupportedScheme(_)));
    }

    #[test]
    fn data_scheme_unsupported() {
        let err = fetch("data:text/html,<h1>hi</h1>").unwrap_err();
        assert!(matches!(err, FetchError::UnsupportedScheme(_)));
    }
}
