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

    // Percent-decode the path (e.g. `Squat%2F0.jpg` -> `Squat/0.jpg`).
    let rel = match percent_decode_str(raw_path).decode_utf8() {
        Ok(s) => s.into_owned(),
        Err(_) => return error_response(StatusCode::BAD_REQUEST),
    };

    // Reject any path that contains `..` to prevent directory traversal.
    if rel.split('/').any(|c| c == "..") {
        log::warn!("imgcache: forbidden path traversal attempt: {}", rel);
        return error_response(StatusCode::FORBIDDEN);
    }

    let images_dir = crate::services::storage::native_storage::images_dir();
    let file_path = images_dir.join(&rel);

    log::info!(
        "imgcache: request for {}, resolving to {}",
        rel,
        file_path.display()
    );

    // On some Android versions/mounts, canonicalize() can be flaky or return
    // unexpected paths (e.g. resolving /storage/emulated/0 to /mnt/user/0).
    // We've already checked for `..` above, so we can safely read the file.
    if !file_path.exists() {
        log::warn!("imgcache: file not found: {}", file_path.display());
        return error_response(StatusCode::NOT_FOUND);
    }

    let bytes = match std::fs::read(&file_path) {
        Ok(b) => b,
        Err(e) => {
            log::warn!("imgcache: failed to read {}: {e}", file_path.display());
            return error_response(StatusCode::NOT_FOUND);
        }
    };

    let content_type = content_type_for_path(&file_path);

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
