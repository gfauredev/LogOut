/// Notification channel ID used for workout alerts (rest-over, duration-reached).
#[cfg(target_os = "android")]
pub const WORKOUT_CHANNEL_ID: &str = "workout_reminders";

/// Android-specific notification channel setup.
///
/// Creates a `NotificationChannel` with `IMPORTANCE_HIGH` so that rest-over
/// and duration-reached alerts bypass Do-Not-Disturb and produce sound and
/// heads-up banners.  Must be called once at app startup (before the first
/// notification is sent); safe to call multiple times.
#[cfg(target_os = "android")]
pub fn setup_notification_channel() {
    use jni::{objects::JObject, JavaVM};
    use ndk_context::android_context;

    let result = (|| -> Result<(), String> {
        let ctx = android_context();
        // SAFETY: raw pointers come from the Android runtime; valid for process lifetime.
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
            .map_err(|e| format!("JavaVM::from_raw: {e}"))?;
        let mut env = vm
            .attach_current_thread()
            .map_err(|e| format!("attach_current_thread: {e}"))?;
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

        // val NOTIFICATION_SERVICE: String = Context.NOTIFICATION_SERVICE
        let notif_service_str = env
            .get_static_field(
                "android/content/Context",
                "NOTIFICATION_SERVICE",
                "Ljava/lang/String;",
            )
            .map_err(|e| format!("get NOTIFICATION_SERVICE: {e}"))?
            .l()
            .map_err(|e| format!("NOTIFICATION_SERVICE as object: {e}"))?;

        // val nm = context.getSystemService(NOTIFICATION_SERVICE) as NotificationManager
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

        // IMPORTANCE_HIGH = 4
        let importance_high = env
            .get_static_field("android/app/NotificationManager", "IMPORTANCE_HIGH", "I")
            .map_err(|e| format!("get IMPORTANCE_HIGH: {e}"))?
            .i()
            .map_err(|e| format!("IMPORTANCE_HIGH as int: {e}"))?;

        // val channel = NotificationChannel(id, name, IMPORTANCE_HIGH)
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

        // nm.createNotificationChannel(channel)
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
        Ok(()) => log::info!(
            "Android notification channel '{}' created",
            WORKOUT_CHANNEL_ID
        ),
        Err(e) => log::warn!("Failed to create Android notification channel: {e}"),
    }
}

#[cfg(not(target_os = "android"))]
pub fn setup_notification_channel() {}

/// Send an Android notification with the given title, body and tag via JNI.
///
/// The tag is used as both the notification tag (for deduplication) and the
/// basis for a stable integer notification ID so that identical tags replace
/// their previous notification rather than stacking.
///
/// Uses a simple text-style `Notification.Builder`.  The small icon falls back
/// to `android.R.drawable.ic_dialog_info` if no app-specific icon is available.
///
/// This function is a no-op when there is no active notification channel (i.e.
/// [`setup_notification_channel`] has not been called yet).
#[cfg(target_os = "android")]
pub fn send_notification(title: &str, body: &str, tag: &str) {
    let title = title.to_string();
    let body = body.to_string();
    let tag = tag.to_string();

    let result = (|| -> Result<(), String> {
        use jni::{objects::JObject, JavaVM};
        use ndk_context::android_context;

        let ctx = android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
            .map_err(|e| format!("JavaVM::from_raw: {e}"))?;
        let mut env = vm
            .attach_current_thread()
            .map_err(|e| format!("attach_current_thread: {e}"))?;
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

        // NotificationManager
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

        // Notification.Builder(context, channelId)
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

        // .setSmallIcon(android.R.drawable.ic_dialog_info)
        // ic_dialog_info has resource id 0x01040014 on all Android versions.
        #[allow(clippy::cast_possible_wrap)]
        let icon_id: i32 = 0x0104_0014_u32 as i32;
        env.call_method(
            &builder,
            "setSmallIcon",
            "(I)Landroid/app/Notification$Builder;",
            &[jni::objects::JValue::Int(icon_id)],
        )
        .map_err(|e| format!("setSmallIcon: {e}"))?;

        // .setContentTitle(title)
        let jtitle = env
            .new_string(&title)
            .map_err(|e| format!("new_string title: {e}"))?;
        env.call_method(
            &builder,
            "setContentTitle",
            "(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;",
            &[(&jtitle).into()],
        )
        .map_err(|e| format!("setContentTitle: {e}"))?;

        // .setContentText(body)
        let jbody = env
            .new_string(&body)
            .map_err(|e| format!("new_string body: {e}"))?;
        env.call_method(
            &builder,
            "setContentText",
            "(Ljava/lang/CharSequence;)Landroid/app/Notification$Builder;",
            &[(&jbody).into()],
        )
        .map_err(|e| format!("setContentText: {e}"))?;

        // .setPriority(PRIORITY_HIGH = 1)
        env.call_method(
            &builder,
            "setPriority",
            "(I)Landroid/app/Notification$Builder;",
            &[jni::objects::JValue::Int(1)],
        )
        .map_err(|e| format!("setPriority: {e}"))?;

        // .setAutoCancel(true)
        env.call_method(
            &builder,
            "setAutoCancel",
            "(Z)Landroid/app/Notification$Builder;",
            &[jni::objects::JValue::Bool(1)],
        )
        .map_err(|e| format!("setAutoCancel: {e}"))?;

        // notification = builder.build()
        let notification = env
            .call_method(&builder, "build", "()Landroid/app/Notification;", &[])
            .map_err(|e| format!("build: {e}"))?
            .l()
            .map_err(|e| format!("Notification obj: {e}"))?;

        // Derive a stable int ID from the tag string so the same tag always
        // replaces its predecessor (ring-mod to keep it positive).
        let notif_id = (tag.len() as i32 * 31_i32)
            .wrapping_add(tag.bytes().fold(0i32, |acc, b| acc.wrapping_add(b as i32)))
            .abs();

        let jtag = env
            .new_string(&tag)
            .map_err(|e| format!("new_string tag: {e}"))?;

        // nm.notify(tag, id, notification)
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
    })();

    match result {
        Ok(()) => log::debug!("Android notification sent: tag={tag}"),
        Err(e) => log::warn!("Failed to send Android notification (tag={tag}): {e}"),
    }
}

#[cfg(not(target_os = "android"))]
pub fn send_notification(_title: &str, _body: &str, _tag: &str) {}
