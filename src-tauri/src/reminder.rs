//! 定时提醒模块
//!
//! 实现定时提醒功能，用于定期弹出复习提醒窗口

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// 提醒管理器
pub struct ReminderManager {
    /// 是否启用提醒
    enabled: AtomicBool,
    /// 提醒间隔（分钟）
    interval_minutes: AtomicU64,
}

impl ReminderManager {
    /// 创建新的提醒管理器
    pub fn new(interval_minutes: u64) -> Self {
        Self {
            enabled: AtomicBool::new(true),
            interval_minutes: AtomicU64::new(interval_minutes),
        }
    }

    /// 启动定时提醒
    pub fn start(&self, app_handle: AppHandle) {
        if !self.enabled.load(Ordering::SeqCst) {
            log::info!("提醒已禁用，跳过启动");
            return;
        }

        let interval = self.interval_minutes.load(Ordering::SeqCst);
        log::info!("启动定时提醒，间隔: {} 分钟", interval);

        let handle = app_handle.clone();
        let interval_secs = interval * 60;

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(interval_secs));

                if let Some(window) = handle.get_webview_window("main") {
                    // 发送提醒事件到前端
                    if let Err(e) = window.emit("review-reminder", ()) {
                        log::error!("发送提醒失败: {}", e);
                    } else {
                        log::info!("复习提醒已触发");
                        // 如果窗口最小化或隐藏，则显示并聚焦
                        if let Ok(visible) = window.is_visible() {
                            if !visible {
                                let _ = window.show();
                            }
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        });
    }

    /// 更新提醒间隔
    pub fn update_interval(&self, minutes: u64) {
        self.interval_minutes.store(minutes, Ordering::SeqCst);
        log::info!("提醒间隔已更新为: {} 分钟", minutes);
    }

    /// 启用/禁用提醒
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
        log::info!("提醒已{}", if enabled { "启用" } else { "禁用" });
    }

    /// 检查提醒是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// 获取当前间隔（分钟）
    pub fn get_interval(&self) -> u64 {
        self.interval_minutes.load(Ordering::SeqCst)
    }
}