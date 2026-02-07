use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, Method, StatusCode, Version},
};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// CORS configuration
pub fn cors_layer() -> tower_http::cors::CorsLayer {
    tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any)
        .max_age(Duration::from_secs(86400))
}

/// Add security headers to response
pub fn add_security_headers(headers: &mut HeaderMap) {
    // Content-Security-Policy
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'self'"));

    // X-Content-Type-Options
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    // X-Frame-Options
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // X-XSS-Protection
    headers.insert(header::X_XSS_PROTECTION, HeaderValue::from_static("1; mode=block"));
}

/// Security headers middleware
pub async fn security_headers_middleware(
    headers: HeaderMap,
    mut request: axum::http::Request<Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(request).await;
    add_security_headers(response.headers_mut());
    response
}

/// Log request timing
fn log_request_timing(
    method: &Method,
    path: &str,
    version: &Version,
    status: StatusCode,
    duration: Duration,
) {
    let status_code = status.as_u16();
    let duration_ms = duration.as_millis();

    let duration_str = if duration_ms < 1 {
        format!("{}μs", duration.as_micros())
    } else if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else {
        format!("{}s", duration.as_secs_f32())
    };

    if status.is_success() {
        debug!("{} {} {:?} {} {}", method, path, version, status_code, duration_str);
    } else if status.is_client_error() {
        warn!("{} {} {:?} {} {}", method, path, version, status_code, duration_str);
    } else {
        warn!("{} {} {:?} {} {}", method, path, version, status_code, duration_str);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_layer_creation() {
        let _cors = cors_layer();
    }

    #[test]
    fn test_security_headers() {
        let mut headers = HeaderMap::new();
        add_security_headers(&mut headers);

        assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));
        assert!(headers.contains_key(header::X_CONTENT_TYPE_OPTIONS));
        assert!(headers.contains_key(header::X_FRAME_OPTIONS));
        assert!(headers.contains_key(header::X_XSS_PROTECTION));
    }
}
