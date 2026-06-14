//! 单词实体

use serde::{Deserialize, Serialize};

/// 单词状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WordStatus {
    New,
    Learning,
    Mastered,
    Skipped,
}

impl WordStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "new" => WordStatus::New,
            "learning" => WordStatus::Learning,
            "mastered" => WordStatus::Mastered,
            "skipped" => WordStatus::Skipped,
            _ => WordStatus::New,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            WordStatus::New => "new",
            WordStatus::Learning => "learning",
            WordStatus::Mastered => "mastered",
            WordStatus::Skipped => "skipped",
        }
    }
}

/// 词义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub id: String,
    pub pos: Option<String>,
    pub definition: String,
    pub example: Option<String>,
}

/// 单词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub id: String,
    pub word: String,
    pub phonetic: Option<String>,
    pub phonetic_audio_url: Option<String>,
    pub definitions: Vec<Definition>,
    pub status: WordStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Word {
    pub fn new(word: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            word,
            phonetic: None,
            phonetic_audio_url: None,
            definitions: Vec::new(),
            status: WordStatus::New,
            created_at: now,
            updated_at: now,
        }
    }
}
