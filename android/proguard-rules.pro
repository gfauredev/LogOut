# Keep Dioxus and NDK bridge classes
-keep class dev.dioxus.** { *; }
-keep class com.gfaure.logworkout.** { *; }
-keep class androidx.webkit.** { *; }

# Allow obfuscation but keep the essential bridge
-keepnames class dev.dioxus.** { *; }
