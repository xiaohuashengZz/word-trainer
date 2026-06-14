//! 领域层 - 核心业务实体

mod word;
mod schedule;
mod review_log;

pub use word::{Definition, Word, WordStatus};
pub use schedule::Schedule;
pub use review_log::ReviewLog;

#[cfg(test)]
mod tests {
    use super::*;

    // ========== WordStatus 测试 ==========
    #[test]
    fn test_word_status_from_str() {
        assert_eq!(WordStatus::from_str("new"), WordStatus::New);
        assert_eq!(WordStatus::from_str("learning"), WordStatus::Learning);
        assert_eq!(WordStatus::from_str("mastered"), WordStatus::Mastered);
        assert_eq!(WordStatus::from_str("skipped"), WordStatus::Skipped);
    }

    #[test]
    fn test_word_status_from_str_case_insensitive() {
        assert_eq!(WordStatus::from_str("NEW"), WordStatus::New);
        assert_eq!(WordStatus::from_str("Learning"), WordStatus::Learning);
        assert_eq!(WordStatus::from_str("MASTERED"), WordStatus::Mastered);
    }

    #[test]
    fn test_word_status_from_str_unknown_defaults_to_new() {
        assert_eq!(WordStatus::from_str("unknown"), WordStatus::New);
        assert_eq!(WordStatus::from_str(""), WordStatus::New);
    }

    #[test]
    fn test_word_status_to_str() {
        assert_eq!(WordStatus::New.to_str(), "new");
        assert_eq!(WordStatus::Learning.to_str(), "learning");
        assert_eq!(WordStatus::Mastered.to_str(), "mastered");
        assert_eq!(WordStatus::Skipped.to_str(), "skipped");
    }

    // ========== Word 测试 ==========
    #[test]
    fn test_word_new() {
        let word = Word::new("hello".to_string());
        assert_eq!(word.word, "hello");
        assert_eq!(word.status, WordStatus::New);
        assert!(!word.id.is_empty());
        assert!(word.definitions.is_empty());
        assert!(word.phonetic.is_none());
        assert!(word.phonetic_audio_url.is_none());
        assert!(word.created_at > 0);
        assert!(word.updated_at > 0);
    }

    #[test]
    fn test_word_new_unique_ids() {
        let word1 = Word::new("hello".to_string());
        let word2 = Word::new("world".to_string());
        assert_ne!(word1.id, word2.id);
    }

    // ========== Definition 测试 ==========
    #[test]
    fn test_definition_creation() {
        let def = Definition {
            id: "1".to_string(),
            pos: Some("n".to_string()),
            definition: "a greeting".to_string(),
            example: Some("Hello, world!".to_string()),
        };
        assert_eq!(def.id, "1");
        assert_eq!(def.pos, Some("n".to_string()));
        assert_eq!(def.definition, "a greeting");
        assert_eq!(def.example, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_definition_without_optional_fields() {
        let def = Definition {
            id: "1".to_string(),
            pos: None,
            definition: "a greeting".to_string(),
            example: None,
        };
        assert!(def.pos.is_none());
        assert!(def.example.is_none());
    }

    // ========== Schedule 测试 ==========
    #[test]
    fn test_schedule_new() {
        let schedule = Schedule::new("word123".to_string());
        assert_eq!(schedule.word_id, "word123");
        assert_eq!(schedule.interval, 60);
        assert_eq!(schedule.ease_factor, 2.5);
        assert_eq!(schedule.repetitions, 0);
        assert!(schedule.last_review.is_none());
        assert!(schedule.next_review > 0);
    }

    // ========== ReviewLog 测试 ==========
    #[test]
    fn test_review_log_new() {
        let log = ReviewLog::new("word123".to_string(), true, "correct answer");
        assert_eq!(log.word_id, "word123");
        assert!(log.is_correct);
        assert_eq!(log.user_answer, Some("correct answer".to_string()));
        assert!(!log.id.is_empty());
        assert!(log.reviewed_at > 0);
    }

    #[test]
    fn test_review_log_incorrect() {
        let log = ReviewLog::new("word123".to_string(), false, "wrong");
        assert!(!log.is_correct);
    }
}
