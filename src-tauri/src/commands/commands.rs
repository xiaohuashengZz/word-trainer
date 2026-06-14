//! Tauri 命令实现

use std::sync::Arc;
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::domain::{Definition, ReviewLog, Schedule, Word, WordStatus};
use crate::infrastructure::Database;
use crate::algorithm::Sm2Algorithm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub is_correct: bool,
    pub matched_definition: Option<Definition>,
    pub correct_definitions: Vec<Definition>,
    pub phonetic: Option<String>,
    pub phonetic_audio_url: Option<String>,
    pub next_review_interval: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_words: i32,
    pub new_words: i32,
    pub learning_words: i32,
    pub mastered_words: i32,
    pub skipped_words: i32,
    pub total_reviews: i32,
    pub total_correct: i32,
    pub total_incorrect: i32,
    pub correct_rate: f64,
}

fn normalize(text: &str) -> String {
    text.trim().to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .trim()
        .to_string()
}

fn check_answer(user_answer: &str, definitions: &[Definition]) -> (bool, Option<Definition>) {
    let normalized = normalize(user_answer);
    for def in definitions {
        let def_normalized = normalize(&def.definition);
        if def_normalized == normalized || def_normalized.contains(&normalized) || normalized.contains(&def_normalized) {
            return (true, Some(def.clone()));
        }
    }
    (false, None)
}

#[tauri::command]
pub async fn list_words(db: State<'_, Arc<Database>>, status: Option<String>, offset: i32, limit: i32) -> Result<Vec<Word>, String> {
    db.list_words(status.as_deref(), offset, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_word(db: State<'_, Arc<Database>>, word_text: String, definitions: Vec<Definition>, phonetic: Option<String>, audio_url: Option<String>) -> Result<Word, String> {
    let mut word = Word::new(word_text);
    word.phonetic = phonetic;
    word.phonetic_audio_url = audio_url;
    word.definitions = definitions.clone();
    db.insert_word(&word).map_err(|e| e.to_string())?;
    for def in definitions {
        db.insert_definition(&word.id, &def).map_err(|e| e.to_string())?;
    }
    let schedule = Schedule::new(word.id.clone());
    db.insert_schedule(&schedule).map_err(|e| e.to_string())?;
    Ok(word)
}

#[tauri::command]
pub async fn delete_word(db: State<'_, Arc<Database>>, word_id: String) -> Result<(), String> {
    db.delete_word(&word_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_statistics(db: State<'_, Arc<Database>>) -> Result<Statistics, String> {
    let (total, correct, incorrect) = db.get_review_stats().map_err(|e| e.to_string())?;
    let (new_count, learning_count, mastered_count, skipped_count) = db.get_word_counts_by_status().map_err(|e| e.to_string())?;
    let correct_rate = if total > 0 { (correct as f64 / total as f64) * 100.0 } else { 0.0 };
    Ok(Statistics { total_words: total, new_words: new_count, learning_words: learning_count, mastered_words: mastered_count, skipped_words: skipped_count, total_reviews: total, total_correct: correct, total_incorrect: incorrect, correct_rate })
}

#[tauri::command]
pub async fn get_next_review_word(db: State<'_, Arc<Database>>) -> Result<Option<Word>, String> {
    db.get_next_review_word().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn submit_review(db: State<'_, Arc<Database>>, word_id: String, user_answer: String) -> Result<ReviewResult, String> {
    let word = db.get_word_by_id(&word_id).map_err(|e| e.to_string())?.ok_or_else(|| "单词不存在".to_string())?;
    let (is_correct, matched_def) = check_answer(&user_answer, &word.definitions);
    let log = ReviewLog::new(word_id.clone(), is_correct, user_answer);
    db.insert_review_log(&log).map_err(|e| e.to_string())?;
    let schedule = db.get_schedule(&word_id).map_err(|e| e.to_string())?;
    let new_schedule = if is_correct { Sm2Algorithm::correct(&schedule) } else { Sm2Algorithm::incorrect(&schedule) };
    db.update_schedule(&new_schedule).map_err(|e| e.to_string())?;
    let new_status = match new_schedule.repetitions { 0..=2 => WordStatus::Learning, _ => WordStatus::Mastered };
    db.update_word_status(&word_id, new_status).map_err(|e| e.to_string())?;
    Ok(ReviewResult { is_correct, matched_definition: matched_def, correct_definitions: word.definitions, phonetic: word.phonetic, phonetic_audio_url: word.phonetic_audio_url, next_review_interval: new_schedule.interval })
}

#[tauri::command]
pub async fn skip_word(db: State<'_, Arc<Database>>, word_id: String) -> Result<(), String> {
    db.update_word_status(&word_id, WordStatus::Skipped).map_err(|e| e.to_string())?;
    db.remove_from_schedule(&word_id).map_err(|e| e.to_string())?;
    db.insert_skip_log(&word_id, "manual").map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn unskip_word(db: State<'_, Arc<Database>>, word_id: String) -> Result<(), String> {
    db.update_word_status(&word_id, WordStatus::Learning).map_err(|e| e.to_string())?;
    let schedule = Schedule::new(word_id);
    db.insert_schedule(&schedule).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_setting(db: State<'_, Arc<Database>>, key: String) -> Result<Option<String>, String> {
    db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(db: State<'_, Arc<Database>>, key: String, value: String) -> Result<(), String> {
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_word_count(db: State<'_, Arc<Database>>) -> Result<i32, String> {
    db.get_word_count().map_err(|e| e.to_string())
}
// 提醒相关命令
use crate::reminder::ReminderManager;

#[tauri::command]
pub async fn start_reminder(
    reminder: State<'_, Arc<ReminderManager>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    reminder.start(app_handle);
    Ok(())
}

#[tauri::command]
pub async fn stop_reminder(reminder: State<'_, Arc<ReminderManager>>) -> Result<(), String> {
    reminder.set_enabled(false);
    Ok(())
}

#[tauri::command]
pub async fn update_reminder_interval(
    reminder: State<'_, Arc<ReminderManager>>,
    minutes: u64,
) -> Result<(), String> {
    reminder.update_interval(minutes);
    Ok(())
}
