//! 复习调度实体

use serde::{Deserialize, Serialize};

/// 复习调度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub word_id: String,
    pub interval: i64,
    pub ease_factor: f64,
    pub repetitions: i32,
    pub next_review: i64,
    pub last_review: Option<i64>,
}

impl Schedule {
    pub fn new(word_id: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            word_id,
            interval: 60,
            ease_factor: 2.5,
            repetitions: 0,
            next_review: now,
            last_review: None,
        }
    }
}