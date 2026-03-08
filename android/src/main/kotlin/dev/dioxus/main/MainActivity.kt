package dev.dioxus.main

import android.os.Bundle
import com.gfaure.logout.BuildConfig

typealias BuildConfig = BuildConfig

class MainActivity : WryActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Provide the internal data directory to the Rust backend
        // before any database operations occur.
        setDataDir(filesDir.absolutePath)
    }

    private external fun setDataDir(dataDir: String)
}
