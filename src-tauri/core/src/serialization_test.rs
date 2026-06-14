//! 序列化测试

#[cfg(test)]
mod tests {
    use crate::{Definition, ReviewLog, Schedule, Word, WordStatus};

    // ========== WordStatus 序列化测试 ==========
    #[test]
    fn test_word_status_serde() {
        let status = WordStatus::New;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"new\"");

        let parsed: WordStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, WordStatus::New);
    }

    #[test]
    fn test_word_status_all_variants_serde() {
        let variants = [
            (WordStatus::New, "new"),
            (WordStatus::Learning, "learning"),
            (WordStatus::Mastered, "mastered"),
            (WordStatus::Skipped, "skipped"),
        ];

        for (status, expected) in variants {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));

            let parsed: WordStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_word_status_from_str_case_insensitive() {
        // from_str 方法是大小写不敏感的
        assert_eq!(WordStatus::from_str("new"), WordStatus::New);
        assert_eq!(WordStatus::from_str("NEW"), WordStatus::New);
        assert_eq!(WordStatus::from_str("New"), WordStatus::New);
        assert_eq!(WordStatus::from_str("nEw"), WordStatus::New);

        assert_eq!(WordStatus::from_str("LEARNING"), WordStatus::Learning);
        assert_eq!(WordStatus::from_str("MASTERED"), WordStatus::Mastered);
        assert_eq!(WordStatus::from_str("Skipped"), WordStatus::Skipped);
    }

    #[test]
    fn test_word_status_from_str_unknown_defaults_to_new() {
        // from_str 方法对未知值返回默认值 New
        assert_eq!(WordStatus::from_str("unknown"), WordStatus::New);
        assert_eq!(WordStatus::from_str("invalid"), WordStatus::New);
        assert_eq!(WordStatus::from_str(""), WordStatus::New);
    }

    #[test]
    fn test_word_status_serde_requires_lowercase() {
        // serde 默认要求小写格式
        let result: Result<WordStatus, _> = serde_json::from_str("\"New\"");
        assert!(result.is_err(), "serde 应该拒绝大写格式");

        let result: Result<WordStatus, _> = serde_json::from_str("\"LEARNING\"");
        assert!(result.is_err(), "serde 应该拒绝大写格式");
    }

    #[test]
    fn test_word_status_serde_rejects_unknown() {
        // serde 对未知值会报错，不会使用默认值
        let result: Result<WordStatus, _> = serde_json::from_str("\"unknown\"");
        assert!(result.is_err(), "serde 应该拒绝未知值");
    }

    // ========== Definition 序列化测试 ==========
    #[test]
    fn test_definition_minimal_serde() {
        let def = Definition {
            id: "1".to_string(),
            pos: None,
            definition: "a test".to_string(),
            example: None,
        };

        let json = serde_json::to_string(&def).unwrap();
        let parsed: Definition = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "1");
        assert_eq!(parsed.definition, "a test");
        assert!(parsed.pos.is_none());
        assert!(parsed.example.is_none());
    }

    #[test]
    fn test_definition_full_serde() {
        let def = Definition {
            id: "123".to_string(),
            pos: Some("n".to_string()),
            definition: "a greeting".to_string(),
            example: Some("Hello!".to_string()),
        };

        let json = serde_json::to_string(&def).unwrap();
        let parsed: Definition = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "123");
        assert_eq!(parsed.pos, Some("n".to_string()));
        assert_eq!(parsed.definition, "a greeting");
        assert_eq!(parsed.example, Some("Hello!".to_string()));
    }

    // ========== Schedule 序列化测试 ==========
    #[test]
    fn test_schedule_serde() {
        let schedule = Schedule::new("word123".to_string());
        let json = serde_json::to_string(&schedule).unwrap();
        let parsed: Schedule = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.word_id, "word123");
        assert_eq!(parsed.interval, 60);
        assert_eq!(parsed.ease_factor, 2.5);
        assert_eq!(parsed.repetitions, 0);
        assert!(parsed.last_review.is_none());
    }

    #[test]
    fn test_schedule_with_history_serde() {
        let mut schedule = Schedule::new("word123".to_string());
        schedule.interval = 600;
        schedule.ease_factor = 2.6;
        schedule.repetitions = 2;
        schedule.last_review = Some(1000000);

        let json = serde_json::to_string(&schedule).unwrap();
        let parsed: Schedule = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.word_id, "word123");
        assert_eq!(parsed.interval, 600);
        assert_eq!(parsed.ease_factor, 2.6);
        assert_eq!(parsed.repetitions, 2);
        assert_eq!(parsed.last_review, Some(1000000));
    }

    // ========== Word 序列化测试 ==========
    #[test]
    fn test_word_minimal_serde() {
        let word = Word::new("test".to_string());
        let json = serde_json::to_string(&word).unwrap();
        let parsed: Word = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.word, "test");
        assert_eq!(parsed.status, WordStatus::New);
        assert!(parsed.phonetic.is_none());
        assert!(parsed.phonetic_audio_url.is_none());
        assert!(parsed.definitions.is_empty());
    }

    #[test]
    fn test_word_full_serde() {
        let mut word = Word::new("hello".to_string());
        word.phonetic = Some("/həˈloʊ/".to_string());
        word.phonetic_audio_url = Some("https://example.com/hello.mp3".to_string());
        word.status = WordStatus::Learning;
        word.definitions.push(Definition {
            id: "1".to_string(),
            pos: Some("n".to_string()),
            definition: "a greeting".to_string(),
            example: Some("Hello, world!".to_string()),
        });

        let json = serde_json::to_string(&word).unwrap();
        let parsed: Word = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.word, "hello");
        assert_eq!(parsed.phonetic, Some("/həˈloʊ/".to_string()));
        assert_eq!(parsed.phonetic_audio_url, Some("https://example.com/hello.mp3".to_string()));
        assert_eq!(parsed.status, WordStatus::Learning);
        assert_eq!(parsed.definitions.len(), 1);
        assert_eq!(parsed.definitions[0].definition, "a greeting");
    }

    #[test]
    fn test_word_with_multiple_definitions_serde() {
        let mut word = Word::new("set".to_string());
        word.definitions.push(Definition {
            id: "1".to_string(),
            pos: Some("v".to_string()),
            definition: "to put something in a place".to_string(),
            example: None,
        });
        word.definitions.push(Definition {
            id: "2".to_string(),
            pos: Some("n".to_string()),
            definition: "a collection of things".to_string(),
            example: None,
        });
        word.definitions.push(Definition {
            id: "3".to_string(),
            pos: Some("adj".to_string()),
            definition: "fixed or predetermined".to_string(),
            example: None,
        });

        let json = serde_json::to_string(&word).unwrap();
        let parsed: Word = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.definitions.len(), 3);
    }

    // ========== ReviewLog 序列化测试 ==========
    #[test]
    fn test_review_log_correct_serde() {
        let log = ReviewLog::new("word123".to_string(), true, "correct answer");
        let json = serde_json::to_string(&log).unwrap();
        let parsed: ReviewLog = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.word_id, "word123");
        assert!(parsed.is_correct);
        assert_eq!(parsed.user_answer, Some("correct answer".to_string()));
        assert!(parsed.reviewed_at > 0);
    }

    #[test]
    fn test_review_log_incorrect_serde() {
        let log = ReviewLog::new("word456".to_string(), false, "wrong");
        let json = serde_json::to_string(&log).unwrap();
        let parsed: ReviewLog = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.word_id, "word456");
        assert!(!parsed.is_correct);
    }

    // ========== JSON 格式测试 ==========
    #[test]
    fn test_word_status_json_format() {
        // 测试前端期望的 JSON 格式
        let status = serde_json::json!("new");
        assert_eq!(status, serde_json::json!("new"));

        let status: WordStatus = serde_json::from_value(serde_json::json!("learning")).unwrap();
        assert_eq!(status, WordStatus::Learning);
    }

    #[test]
    fn test_word_json_structure() {
        let json_str = r#"{
            "id": "test-id",
            "word": "hello",
            "phonetic": "/həˈloʊ/",
            "phonetic_audio_url": null,
            "definitions": [
                {
                    "id": "d1",
                    "pos": "n",
                    "definition": "a greeting",
                    "example": "Hello!"
                }
            ],
            "status": "new",
            "created_at": 1000000,
            "updated_at": 1000000
        }"#;

        let word: Word = serde_json::from_str(json_str).unwrap();
        assert_eq!(word.word, "hello");
        assert_eq!(word.status, WordStatus::New);
        assert_eq!(word.definitions.len(), 1);
    }

    #[test]
    fn test_schedule_json_structure() {
        let json_str = r#"{
            "word_id": "word-123",
            "interval": 600,
            "ease_factor": 2.5,
            "repetitions": 3,
            "next_review": 2000000,
            "last_review": 1000000
        }"#;

        let schedule: Schedule = serde_json::from_str(json_str).unwrap();
        assert_eq!(schedule.word_id, "word-123");
        assert_eq!(schedule.interval, 600);
        assert_eq!(schedule.repetitions, 3);
    }

    #[test]
    fn test_review_log_json_structure() {
        let json_str = r#"{
            "id": "log-123",
            "word_id": "word-456",
            "is_correct": true,
            "user_answer": "hello",
            "reviewed_at": 3000000
        }"#;

        let log: ReviewLog = serde_json::from_str(json_str).unwrap();
        assert_eq!(log.word_id, "word-456");
        assert!(log.is_correct);
    }
}
