/// Custom `imgcache://` protocol handler for serving locally-cached exercise images.
///
/// Dioxus on Android runs inside a WebView whose origin is `https://dioxus.index.html/`.
/// Android's WebView security policy blocks loading `file://` resources from an
/// `https://` origin ("Not allowed to load local resource").  Registering a named
/// custom protocol is the standard wry/Dioxus workaround: the handler runs in the
/// native process and has unrestricted filesystem access.
///
/// URL format: `imgcache://localhost/<relative-path>`
///
/// The relative path is resolved against `data_dir()/images/` – the directory where
/// `download_db_images` stores downloaded exercise images and where user-uploaded
/// images are copied via `local:` prefixed keys.
///
/// Security: path-traversal attempts (any `..` component after URL-decoding) are
/// rejected with a 404 response.  The resolved absolute path is also verified to be
/// inside the images directory before the file is read.
#[cfg(feature = "mobile-platform")]
pub(crate) fn handle_imgcache_request(
    request: dioxus::mobile::wry::http::Request<Vec<u8>>,
) -> dioxus::mobile::wry::http::Response<std::borrow::Cow<'static, [u8]>> {
    use dioxus::mobile::wry::http::{Response, StatusCode};
    use percent_encoding::percent_decode_str;
    use std::borrow::Cow;

    // Strip the leading `/` from the URI path.
    let raw_path = request.uri().path().trim_start_matches('/');

    // URL-decode the path so that percent-encoded characters are handled correctly.
    let rel = match percent_decode_str(raw_path).decode_utf8() {
        Ok(s) => s.into_owned(),
        Err(_) => return error_response(StatusCode::BAD_REQUEST),
    };

    // Reject any path that contains `..` to prevent directory traversal.
    if rel.split('/').any(|c| c == "..") {
        return error_response(StatusCode::FORBIDDEN);
    }

    let images_dir = crate::services::storage::native_storage::data_dir().join("images");
    let file_path = images_dir.join(&rel);

    // Canonicalize to resolve symlinks and verify the path stays within images_dir.
    let canonical = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return error_response(StatusCode::NOT_FOUND),
    };
    let images_canonical = match images_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR),
    };
    if !canonical.starts_with(&images_canonical) {
        return error_response(StatusCode::FORBIDDEN);
    }

    let bytes = match std::fs::read(&canonical) {
        Ok(b) => b,
        Err(_) => return error_response(StatusCode::NOT_FOUND),
    };

    let content_type = content_type_for_path(&canonical);
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        // Images are immutable once cached – let the WebView cache them aggressively.
        .header("Cache-Control", "max-age=31536000, immutable")
        .body(Cow::Owned(bytes))
        .unwrap()
}

/// Returns an empty response with the given status code.
#[cfg(feature = "mobile-platform")]
fn error_response(
    status: dioxus::mobile::wry::http::StatusCode,
) -> dioxus::mobile::wry::http::Response<std::borrow::Cow<'static, [u8]>> {
    use std::borrow::Cow;
    dioxus::mobile::wry::http::Response::builder()
        .status(status)
        .body(Cow::Borrowed(b"" as &[u8]))
        .unwrap()
}

/// Infers the MIME type from the file extension.
#[cfg(feature = "mobile-platform")]
fn content_type_for_path(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}
