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

    let response = match ureq::get(url)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) Quint/0.1.0")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,text/css,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .call()
    {
        Ok(res) => res,
        Err(ureq::Error::StatusCode(code)) => {
            return Err(FetchError::HttpStatus {
                status: code,
                url: url.to_string(),
            });
        }
        Err(e) => return Err(FetchError::Network(e)),
    };

    let status = response.status();
    let status_code = status.as_u16();
    // In case the above match didn't catch it or ureq is configured otherwise
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

    let mut body_bytes = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(MAX_BODY_SIZE)
        .read_to_end(&mut body_bytes)?;
    let body = String::from_utf8_lossy(&body_bytes).into_owned();

    Ok(FetchResponse {
        final_url: url.to_string(),
        status: status_code,
        content_type,
        body,
    })
}

/// Resolves a relative or absolute URL against a base URL according to standard URL resolution rules.
pub fn resolve_url(base: &str, relative: &str) -> String {
    let rel_trimmed = relative.trim();
    if rel_trimmed.starts_with("http://")
        || rel_trimmed.starts_with("https://")
        || rel_trimmed.starts_with("data:")
        || rel_trimmed.starts_with("file://")
    {
        return rel_trimmed.to_string();
    }

    let (scheme, rest) = if let Some(idx) = base.find("://") {
        (&base[..idx + 3], &base[idx + 3..])
    } else {
        ("https://", base)
    };

    // Protocol-relative URL: //example.com/foo
    if rel_trimmed.starts_with("//") {
        let s = scheme.split(':').next().unwrap_or("https");
        return format!("{}:{}", s, rel_trimmed);
    }

    let host_and_path = rest;
    let (host, path) = match host_and_path.find('/') {
        Some(idx) => (&host_and_path[..idx], &host_and_path[idx..]),
        None => (host_and_path, "/"),
    };

    // Root-relative URL: /foo/bar
    if rel_trimmed.starts_with('/') {
        return format!("{}{}{}", scheme, host, rel_trimmed);
    }

    // Path-relative URL: foo.css, ./foo.css, ../foo.css
    let base_dir = match path.rfind('/') {
        Some(idx) => &path[..=idx],
        None => "/",
    };

    let mut parts: Vec<&str> = base_dir
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    for segment in rel_trimmed.split('/') {
        if segment == "." || segment.is_empty() {
            continue;
        } else if segment == ".." {
            parts.pop();
        } else {
            parts.push(segment);
        }
    }

    format!("{}{}/{}", scheme, host, parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_url() {
        assert_eq!(
            resolve_url("https://example.com/path/index.html", "style.css"),
            "https://example.com/path/style.css"
        );
        assert_eq!(
            resolve_url("https://example.com/path/index.html", "/global.css"),
            "https://example.com/global.css"
        );
        assert_eq!(
            resolve_url("https://example.com/a/b/c", "../other.css"),
            "https://example.com/a/other.css"
        );
        assert_eq!(
            resolve_url("https://example.com", "//cdn.org/lib.js"),
            "https://cdn.org/lib.js"
        );
        assert_eq!(
            resolve_url("http://example.com/index.html", "https://other.com/a.css"),
            "https://other.com/a.css"
        );
    }

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
