//! 英语单词学习助手 - Tauri 后端

use std::sync::Arc;
use tauri::{menu::{Menu, MenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, Emitter, Manager};

mod algorithm;
mod commands;
mod domain;
mod infrastructure;
mod reminder;

use infrastructure::Database;

fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis().init();
    log::info!("英语单词学习助手启动中...");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let review_item = MenuItem::with_id(app, "review", "开始复习", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &review_item, &quit_item])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("英语单词学习助手")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => { if let Some(w) = app.get_webview_window("main") { let _ = w.show(); let _ = w.set_focus(); } }
                "review" => { if let Some(w) = app.get_webview_window("main") { let _ = w.emit("start-review", ()); } }
                "quit" => { app.exit(0); }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") { let _ = w.show(); let _ = w.set_focus(); }
            }
        })
        .build(app)?;
    log::info!("系统托盘已设置");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            let db = Database::new(data_dir).expect("无法初始化数据库");
            app.manage(Arc::new(db));

            // 初始化并启动定时提醒
            let reminder_manager = reminder::ReminderManager::new(30); // 默认30分钟
            app.manage(Arc::new(reminder_manager));

            if let Err(e) = setup_tray(app) { log::error!("设置托盘失败: {}", e); }
            log::info!("应用初始化完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_words, commands::add_word, commands::delete_word,
            commands::get_statistics, commands::get_next_review_word, commands::submit_review,
            commands::skip_word, commands::unskip_word, commands::get_setting,
            commands::set_setting, commands::get_word_count,
            commands::start_reminder, commands::stop_reminder, commands::update_reminder_interval,
        ])
        .run(tauri::generate_context!())
        .expect("运行应用时发生错误");
}