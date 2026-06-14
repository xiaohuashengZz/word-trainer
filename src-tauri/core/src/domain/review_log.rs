//! 复习日志实体

use serde::{Deserialize, Serialize};

/// 复习日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewLog {
    pub id: String,
    pub word_id: String,
    pub is_correct: bool,
    pub user_answer: Option<String>,
    pub reviewed_at: i64,
}

impl ReviewLog {
    pub fn new(word_id: String, is_correct: bool, user_answer: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            word_id,
            is_correct,
            user_answer: Some(user_answer.into()),
            reviewed_at: chrono::Utc::now().timestamp_millis(),
        }
    }
}
