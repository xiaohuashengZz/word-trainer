//! Mock 数据库用于测试
#![allow(unused_imports)]

use std::collections::HashMap;
use crate::domain::{Definition, Schedule, Word, WordStatus, ReviewLog};

/// Mock 数据库结构
#[derive(Debug, Clone, Default)]
pub struct MockDatabase {
    words: HashMap<String, Word>,
    schedules: HashMap<String, Schedule>,
    review_logs: Vec<ReviewLog>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    // ========== 单词操作 ==========
    pub fn insert_word(&mut self, word: &Word) {
        self.words.insert(word.id.clone(), word.clone());
    }

    pub fn get_word(&self, id: &str) -> Option<Word> {
        self.words.get(id).cloned()
    }

    pub fn update_word_status(&mut self, word_id: &str, status: WordStatus) {
        if let Some(word) = self.words.get_mut(word_id) {
            word.status = status;
        }
    }

    pub fn delete_word(&mut self, word_id: &str) {
        self.words.remove(word_id);
        self.schedules.remove(word_id);
        // 删除该单词的所有复习日志
        self.review_logs.retain(|log| log.word_id != word_id);
    }

    // ========== 调度操作 ==========
    pub fn insert_schedule(&mut self, schedule: &Schedule) {
        self.schedules.insert(schedule.word_id.clone(), schedule.clone());
    }

    pub fn get_schedule(&self, word_id: &str) -> Option<Schedule> {
        self.schedules.get(word_id).cloned()
    }

    pub fn update_schedule(&mut self, schedule: &Schedule) {
        self.schedules.insert(schedule.word_id.clone(), schedule.clone());
    }

    pub fn remove_schedule(&mut self, word_id: &str) {
        self.schedules.remove(word_id);
    }

    // ========== 复习日志操作 ==========
    pub fn insert_review_log(&mut self, log: &ReviewLog) {
        self.review_logs.push(log.clone());
    }

    pub fn get_review_logs(&self, word_id: &str) -> Vec<&ReviewLog> {
        self.review_logs.iter().filter(|l| l.word_id == word_id).collect()
    }

    // ========== 统计操作 ==========
    pub fn get_word_count(&self) -> usize {
        self.words.len()
    }

    pub fn get_review_stats(&self) -> (usize, usize) {
        let total = self.review_logs.len();
        let correct = self.review_logs.iter().filter(|l| l.is_correct).count();
        (total, correct)
    }

    pub fn get_next_review_word(&self) -> Option<Word> {
        let now = chrono::Utc::now().timestamp_millis();
        self.schedules
            .iter()
            .filter(|(word_id, schedule)| {
                schedule.next_review <= now &&
                self.words.get(*word_id).map(|w| w.status != WordStatus::Skipped).unwrap_or(false)
            })
            .min_by_key(|(_, schedule)| schedule.next_review)
            .and_then(|(word_id, _)| self.words.get(word_id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_word(word_text: &str) -> Word {
        let mut word = Word::new(word_text.to_string());
        word.definitions.push(Definition {
            id: uuid::Uuid::new_v4().to_string(),
            pos: Some("n".to_string()),
            definition: format!("definition of {}", word_text),
            example: None,
        });
        word
    }

    #[test]
    fn test_mock_db_insert_and_get_word() {
        let mut db = MockDatabase::new();
        let word = create_test_word("hello");

        db.insert_word(&word);
        let retrieved = db.get_word(&word.id);

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().word, "hello");
    }

    #[test]
    fn test_mock_db_update_word_status() {
        let mut db = MockDatabase::new();
        let word = create_test_word("test");

        db.insert_word(&word);
        assert_eq!(db.get_word(&word.id).unwrap().status, WordStatus::New);

        db.update_word_status(&word.id, WordStatus::Learning);
        assert_eq!(db.get_word(&word.id).unwrap().status, WordStatus::Learning);
    }

    #[test]
    fn test_mock_db_delete_word() {
        let mut db = MockDatabase::new();
        let word = create_test_word("test");

        db.insert_word(&word);
        assert!(db.get_word(&word.id).is_some());

        db.delete_word(&word.id);
        assert!(db.get_word(&word.id).is_none());
    }

    #[test]
    fn test_mock_db_schedule_operations() {
        let mut db = MockDatabase::new();
        let word = create_test_word("test");
        db.insert_word(&word);

        let schedule = Schedule::new(word.id.clone());
        db.insert_schedule(&schedule);

        let retrieved = db.get_schedule(&word.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().repetitions, 0);
    }

    #[test]
    fn test_mock_db_review_log() {
        let mut db = MockDatabase::new();
        let word = create_test_word("test");
        db.insert_word(&word);

        let log = ReviewLog::new(word.id.clone(), true, "correct");
        db.insert_review_log(&log);

        let logs = db.get_review_logs(&word.id);
        assert_eq!(logs.len(), 1);
        assert!(logs[0].is_correct);
    }

    #[test]
    fn test_mock_db_review_stats() {
        let mut db = MockDatabase::new();
        let word = create_test_word("test");
        db.insert_word(&word);

        db.insert_review_log(&ReviewLog::new(word.id.clone(), true, "a"));
        db.insert_review_log(&ReviewLog::new(word.id.clone(), true, "b"));
        db.insert_review_log(&ReviewLog::new(word.id.clone(), false, "c"));

        let (total, correct) = db.get_review_stats();
        assert_eq!(total, 3);
        assert_eq!(correct, 2);
    }

    #[test]
    fn test_mock_db_next_review_word() {
        let mut db = MockDatabase::new();
        let word1 = create_test_word("word1");
        let word2 = create_test_word("word2");
        db.insert_word(&word1);
        db.insert_word(&word2);

        let mut schedule1 = Schedule::new(word1.id.clone());
        schedule1.next_review = chrono::Utc::now().timestamp_millis() - 1000; // 过期
        let schedule2 = Schedule::new(word2.id.clone());

        db.insert_schedule(&schedule1);
        db.insert_schedule(&schedule2);

        let next = db.get_next_review_word();
        assert!(next.is_some());
        assert_eq!(next.unwrap().word, "word1");
    }

    #[test]
    fn test_mock_db_skipped_word_not_returned() {
        let mut db = MockDatabase::new();
        let word = create_test_word("test");
        db.insert_word(&word);

        db.update_word_status(&word.id, WordStatus::Skipped);

        let schedule = Schedule::new(word.id.clone());
        db.insert_schedule(&schedule);

        let next = db.get_next_review_word();
        assert!(next.is_none());
    }
}
