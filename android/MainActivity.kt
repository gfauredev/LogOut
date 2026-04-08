package dev.dioxus.main

import android.os.Bundle
import android.view.WindowManager
import android.webkit.WebView

typealias BuildConfig = re.guilhemfau.logout.BuildConfig

/// Custom MainActivity that extends WryActivity with two Android-specific
/// improvements:
///
/// 1. `windowSoftInputMode = SOFT_INPUT_ADJUST_RESIZE` — tells Android to
///    resize the WebView (rather than pan/scroll the whole window) when the
///    soft keyboard opens.  Without this flag the viewport shifts and the
///    WebView re-computes layout, which causes the cursor to jump to the end
///    of text fields on every keystroke.
///
/// 2. WebView.onPause() / onResume() — the Android WebView must be explicitly
///    paused/resumed via its own lifecycle methods (separate from the Activity
///    lifecycle calls that WryActivity already makes via the Rust `pause()`/
///    `resume()` JNI functions).  Without this the JavaScript engine keeps
///    running in the background, accumulating pending layout frames; when the
///    app returns to the foreground the WebView tries to flush ~30 seconds of
///    buffered work at once, causing the app to appear frozen for that period.
class MainActivity : WryActivity() {
    private var rustWebView: WebView? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        // Resize the view when the soft keyboard opens so that the WebView
        // scrolls content into view and the caret stays in its correct position.
        @Suppress("DEPRECATION")
        window.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)
        super.onCreate(savedInstanceState)
    }

    override fun onWebViewCreate(webView: WebView) {
        rustWebView = webView
    }

    override fun onResume() {
        super.onResume()
        rustWebView?.onResume()
    }

    override fun onPause() {
        rustWebView?.onPause()
        super.onPause()
    }
}
