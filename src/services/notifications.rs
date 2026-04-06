/// Global notifications service for the `LogOut` application.
///
/// Handles high-importance alerts (rest-over, duration-reached) by bridging to
/// platform-native APIs:
/// - **Android**: JNI calls to `NotificationManager` (foreground sounds/vibrate).
/// - **Web**: PWA `ServiceWorkerRegistration.showNotification` (sound/vibrate support).
/// - **Desktop**: (TODO) local system notification implementation.

/// Notification channel ID used for workout alerts on Android.
#[cfg(target_os = "android")]
pub const WORKOUT_CHANNEL_ID: &str = "workout_reminders";

/// Returns `true` if notification permission has been granted by the user.
///
/// On Android, this checks `NotificationManager.areNotificationsEnabled()`.
/// On Web, this checks `web_sys::Notification::permission()`.
pub fn is_notification_permission_granted() -> bool {
    #[cfg(target_os = "android")]
    {
        check_android_notification_permission().unwrap_or_else(|e| {
            log::warn!("Failed to check Android notification permission: {e}");
            false
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::NotificationPermission;
        web_sys::Notification::permission() == NotificationPermission::Granted
    }
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    {
        true
    }
}

#[cfg(target_os = "android")]
fn check_android_notification_permission() -> Result<bool, String> {
    use jni::{objects::JObject, JavaVM};
    use ndk_context::android_context;

    let ctx = android_context();
    if ctx.vm().is_null() || ctx.context().is_null() {
        return Err("Android context not available".into());
    }
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
        .map_err(|e| format!("JavaVM::from_raw: {e}"))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("attach_current_thread: {e}"))?;
    let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

    let notif_service_str = env
        .get_static_field(
            "android/content/Context",
            "NOTIFICATION_SERVICE",
            "Ljava/lang/String;",
        )
        .map_err(|e| format!("get NOTIFICATION_SERVICE: {e}"))?
        .l()
        .map_err(|e| format!("NOTIFICATION_SERVICE obj: {e}"))?;
    let nm = env
        .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[(&notif_service_str).into()],
        )
        .map_err(|e| format!("getSystemService: {e}"))?
        .l()
        .map_err(|e| format!("NotificationManager obj: {e}"))?;

    let enabled = env
        .call_method(&nm, "areNotificationsEnabled", "()Z", &[])
        .map_err(|e| format!("areNotificationsEnabled: {e}"))?
        .z()
        .map_err(|e| format!("areNotificationsEnabled as bool: {e}"))?;

    log::debug!("Android notification permission: enabled={}", enabled);
    Ok(enabled)
}

/// Opens the system notification settings for the application.
#[cfg(target_os = "android")]
pub fn open_notification_settings() {
    use jni::{objects::JObject, JavaVM};
    use ndk_context::android_context;

    let result = (|| -> Result<(), String> {
        let ctx = android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
            .map_err(|e| format!("JavaVM::from_raw: {e}"))?;
        let mut env = vm
            .attach_current_thread()
            .map_err(|e| format!("attach_current_thread: {e}"))?;
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

        let intent_cls = env
            .find_class("android/content/Intent")
            .map_err(|e| format!("find Intent class: {e}"))?;
        let action = env
            .new_string("android.settings.APP_NOTIFICATION_SETTINGS")
            .map_err(|e| format!("new_string action: {e}"))?;
        let intent = env
            .new_object(&intent_cls, "(Ljava/lang/String;)V", &[(&action).into()])
            .map_err(|e| format!("new Intent: {e}"))?;

        let pkg_name = env
            .call_method(&activity, "getPackageName", "()Ljava/lang/String;", &[])
            .map_err(|e| format!("getPackageName: {e}"))?
            .l()
            .map_err(|e| format!("getPackageName as object: {e}"))?;

        let extra_pkg = env
            .new_string("android.provider.extra.APP_PACKAGE")
            .map_err(|e| format!("new_string extra_pkg: {e}"))?;

        env.call_method(
            &intent,
            "putExtra",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
            &[(&extra_pkg).into(), (&pkg_name).into()],
        )
        .map_err(|e| format!("putExtra: {e}"))?;

        env.call_method(
            &activity,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[(&intent).into()],
        )
        .map_err(|e| format!("startActivity: {e}"))?;

        Ok(())
    })();

    if let Err(e) = result {
        log::error!("Failed to open Android notification settings: {e}");
    }
}

/// Cross-platform notification dispatch.
///
/// Dispatches the request to the best available platform-specific implementation.
/// On platforms without an implementation yet, this is a no-op.
pub fn send_notification(title: &str, body: &str, tag: &str) {
    #[cfg(target_os = "android")]
    {
        match try_send_android_notification(title, body, tag) {
            Ok(()) => log::debug!("Android notification sent: tag={tag}"),
            Err(e) => log::warn!("Failed to send Android notification (tag={tag}): {e}"),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        send_web_notification(title, body, tag);
    }
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    {
        // Suppress unused warnings for other native platforms (e.g. desktop).
        let _ = (title, body, tag);
        log::info!("Notification [tag={tag}]: {title} - {body}");
    }
}

/// Android-specific notification channel setup.
///
/// Creates a `NotificationChannel` with `IMPORTANCE_HIGH`.  Must be called once
/// at app startup (before the first notification is sent).
pub fn setup_notification_channel() {
    #[cfg(target_os = "android")]
    {
        use jni::{objects::JObject, JavaVM};
        use ndk_context::android_context;

        let result = (|| -> Result<(), String> {
            let ctx = android_context();
            let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
                .map_err(|e| format!("JavaVM::from_raw: {e}"))?;
            let mut env = vm
                .attach_current_thread()
                .map_err(|e| format!("attach_current_thread: {e}"))?;
            let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

            let notif_service_str = env
                .get_static_field(
                    "android/content/Context",
                    "NOTIFICATION_SERVICE",
                    "Ljava/lang/String;",
                )
                .map_err(|e| format!("get NOTIFICATION_SERVICE: {e}"))?
                .l()
                .map_err(|e| format!("NOTIFICATION_SERVICE as object: {e}"))?;

            let nm = env
                .call_method(
                    &activity,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[(&notif_service_str).into()],
                )
                .map_err(|e| format!("getSystemService: {e}"))?
                .l()
                .map_err(|e| format!("NotificationManager as object: {e}"))?;

            let importance_high = env
                .get_static_field("android/app/NotificationManager", "IMPORTANCE_HIGH", "I")
                .map_err(|e| format!("get IMPORTANCE_HIGH: {e}"))?
                .i()
                .map_err(|e| format!("IMPORTANCE_HIGH as int: {e}"))?;

            let channel_id = env
                .new_string(WORKOUT_CHANNEL_ID)
                .map_err(|e| format!("new_string channel_id: {e}"))?;
            let channel_name = env
                .new_string("Workout Reminders")
                .map_err(|e| format!("new_string channel_name: {e}"))?;
            let channel = env
                .new_object(
                    "android/app/NotificationChannel",
                    "(Ljava/lang/String;Ljava/lang/CharSequence;I)V",
                    &[
                        (&channel_id).into(),
                        (&channel_name).into(),
                        jni::objects::JValue::Int(importance_high),
                    ],
                )
                .map_err(|e| format!("new NotificationChannel: {e}"))?;

            env.call_method(
                &nm,
                "createNotificationChannel",
                "(Landroid/app/NotificationChannel;)V",
                &[(&channel).into()],
            )
            .map_err(|e| format!("createNotificationChannel: {e}"))?;

            Ok(())
        })();

        match result {
            Ok(()) => log::info!("Android notification channel '{WORKOUT_CHANNEL_ID}' created"),
            Err(e) => log::warn!("Failed to create Android notification channel: {e}"),
        }
    }
}

/// JNI implementation of Android notification delivery.
#[cfg(target_os = "android")]
fn try_send_android_notification(title: &str, body: &str, tag: &str) -> Result<(), String> {
    use jni::{objects::JObject, JavaVM};
    use ndk_context::android_context;

    let ctx = android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
        .map_err(|e| format!("JavaVM::from_raw: {e}"))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("attach_current_thread: {e}"))?;
    let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

    let notif_service_str = env
        .get_static_field(
            "android/content/Context",
            "NOTIFICATION_SERVICE",
            "Ljava/lang/String;",
        )
        .map_err(|e| format!("get NOTIFICATION_SERVICE: {e}"))?
        .l()
        .map_err(|e| format!("NOTIFICATION_SERVICE obj: {e}"))?;
    let nm = env
        .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[(&notif_service_str).into()],
        )
        .map_err(|e| format!("getSystemService: {e}"))?
        .l()
        .map_err(|e| format!("NotificationManager obj: {e}"))?;

    let channel_id_js = env
        .new_string(WORKOUT_CHANNEL_ID)
        .map_err(|e| format!("new_string channel_id: {e}"))?;
    let builder = env
        .new_object(
            "android/app/Notification$Builder",
            "(Landroid/content/Context;Ljava/lang/String;)V",
            &[(&activity).into(), (&channel_id_js).into()],
        )
        .map_err(|e| format!("new Notification.Builder: {e}"))?;

    #[allow(clippy::cast_possible_wrap)]
    let icon_id: i32 = 0x0104_0014_u32 as i32;
    env.call_method(
        &builder,
        "setSmallIcon",
        "(I)Landroid/app/Notification$Builder;",
        &[jni::objects::JValue::Int(icon_id)],
    )
    .map_err(|e| format!("setSmallIcon: {e}"))?;

    let jtitle = env
        .new_string(title)
        .map_err(|e| format!("new_string title: {e}"))?;
    env.call_method(
        &builder,
        "setContentTitle",
        "(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;",
        &[(&jtitle).into()],
    )
    .map_err(|e| format!("setContentTitle: {e}"))?;

    let jbody = env
        .new_string(body)
        .map_err(|e| format!("new_string body: {e}"))?;
    env.call_method(
        &builder,
        "setContentText",
        "(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;",
        &[(&jbody).into()],
    )
    .map_err(|e| format!("setContentText: {e}"))?;

    env.call_method(
        &builder,
        "setPriority",
        "(I)Landroid/app/Notification$Builder;",
        &[jni::objects::JValue::Int(1)],
    )
    .map_err(|e| format!("setPriority: {e}"))?;

    env.call_method(
        &builder,
        "setAutoCancel",
        "(Z)Landroid/app/Notification$Builder;",
        &[jni::objects::JValue::Bool(1)],
    )
    .map_err(|e| format!("setAutoCancel: {e}"))?;

    let notification = env
        .call_method(&builder, "build", "()Landroid/app/Notification;", &[])
        .map_err(|e| format!("build: {e}"))?
        .l()
        .map_err(|e| format!("Notification obj: {e}"))?;

    let tag_len = i32::try_from(tag.len()).unwrap_or(i32::MAX);
    let notif_id = (tag_len.wrapping_mul(31_i32))
        .wrapping_add(
            tag.bytes()
                .fold(0i32, |acc, b| acc.wrapping_add(i32::from(b))),
        )
        .abs();

    let jtag = env
        .new_string(tag)
        .map_err(|e| format!("new_string tag: {e}"))?;

    env.call_method(
        &nm,
        "notify",
        "(Ljava/lang/String;ILandroid/app/Notification;)V",
        &[
            (&jtag).into(),
            jni::objects::JValue::Int(notif_id),
            (&notification).into(),
        ],
    )
    .map_err(|e| format!("notify: {e}"))?;

    Ok(())
}

/// Web-specific notification delivery using the browser's ServiceWorker API.
#[cfg(target_arch = "wasm32")]
fn send_web_notification(title: &str, body: &str, tag: &str) {
    use web_sys::{NotificationOptions, NotificationPermission};
    if web_sys::Notification::permission() != NotificationPermission::Granted {
        return;
    }
    let title = title.to_string();
    let body = body.to_string();
    let tag = tag.to_string();
    let opts = NotificationOptions::new();
    opts.set_body(&body);
    opts.set_tag(&tag);
    let vibrate = serde_wasm_bindgen::to_value(&[200u32, 100, 200]).ok();
    if let Some(v) = vibrate {
        opts.set_vibrate(&v);
    }
    wasm_bindgen_futures::spawn_local(async move {
        if let Some(window) = web_sys::window() {
            let sw = window.navigator().service_worker();
            if let Ok(ready_promise) = sw.ready() {
                match wasm_bindgen_futures::JsFuture::from(ready_promise).await {
                    Ok(reg_val) => {
                        let reg: web_sys::ServiceWorkerRegistration = reg_val.into();
                        let _ = reg.show_notification_with_options(&title, &opts);
                        return;
                    }
                    Err(e) => {
                        log::warn!("Service worker not ready for notification: {:?}", e);
                    }
                }
            }
        }
        let _ = web_sys::Notification::new_with_options(&title, &opts);
    });
}
