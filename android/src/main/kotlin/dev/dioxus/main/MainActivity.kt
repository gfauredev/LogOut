package dev.dioxus.main

import android.os.Bundle
import com.gfaure.logout.BuildConfig

typealias BuildConfig = BuildConfig

class MainActivity : WryActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        // Provide the internal data directory to the Rust backend
        // BEFORE super.onCreate() which initialises the Dioxus/Wry
        // runtime and may trigger database operations immediately.
        setDataDir(filesDir.absolutePath)
        super.onCreate(savedInstanceState)
    }

    private external fun setDataDir(dataDir: String)
}
