package dev.dioxus.main;
import android.os.Bundle;
import androidx.appcompat.app.AppCompatDelegate;
import com.gfaure.logout.BuildConfig;
typealias BuildConfig = BuildConfig;
class MainActivity : WryActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        // Pass the app-internal data directory to Rust before the app starts,
        // ensuring the SQLite database is created in a writable location and
        // not in a read-only system directory (fixes IO error on Android).
        setDataDir(filesDir.absolutePath)
        // Follow the system dark-mode setting so that the WebView correctly
        // reports prefers-color-scheme: dark when the user has dark mode on.
        AppCompatDelegate.setDefaultNightMode(AppCompatDelegate.MODE_NIGHT_FOLLOW_SYSTEM)
        super.onCreate(savedInstanceState)
    }
    companion object {
        init {
            System.loadLibrary("dioxusmain")
        }
        // Declared as a static native method to match the Rust JNI export:
        // Java_dev_dioxus_main_MainActivity_setDataDir (JClass second param).
        @JvmStatic
        external fun setDataDir(path: String)
    }
}
