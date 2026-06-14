//! 集成测试 - 完整业务流程链路测试

#[cfg(test)]
mod tests {
    use crate::domain::{Definition, ReviewLog, Schedule, Word, WordStatus};
    use crate::mock_db::MockDatabase;
    use crate::Sm2Algorithm;

    // ========== 单词完整生命周期测试 ==========
    // 测试一个单词从创建到掌握的全过程

    fn create_test_word_with_definitions(db: &mut MockDatabase, word_text: &str, definitions: Vec<Definition>) -> Word {
        let mut word = Word::new(word_text.to_string());
        word.definitions = definitions;
        db.insert_word(&word);
        let schedule = Schedule::new(word.id.clone());
        db.insert_schedule(&schedule);
        word
    }

    #[test]
    fn test_word_lifecycle_new_to_mastered() {
        // 模拟一个单词从新单词到掌握的过程
        let mut db = MockDatabase::new();

        // 1. 创建新单词
        let definitions = vec![Definition {
            id: "1".to_string(),
            pos: Some("n".to_string()),
            definition: "a greeting".to_string(),
            example: Some("Hello!".to_string()),
        }];
        let mut word = create_test_word_with_definitions(&mut db, "hello", definitions);

        assert_eq!(word.status, WordStatus::New);

        // 2. 第一次复习：正确
        let schedule = db.get_schedule(&word.id).unwrap();
        let new_schedule = Sm2Algorithm::correct(&schedule);
        db.update_schedule(&new_schedule);
        word.status = match new_schedule.repetitions {
            0..=2 => WordStatus::Learning,
            _ => WordStatus::Mastered,
        };
        db.update_word_status(&word.id, word.status);

        let log = ReviewLog::new(word.id.clone(), true, "a greeting");
        db.insert_review_log(&log);

        assert_eq!(word.status, WordStatus::Learning);
        assert_eq!(new_schedule.repetitions, 1);
        assert_eq!(new_schedule.interval, 60);

        // 3. 第二次复习：正确
        let schedule = db.get_schedule(&word.id).unwrap();
        let new_schedule = Sm2Algorithm::correct(&schedule);
        db.update_schedule(&new_schedule);
        word.status = match new_schedule.repetitions {
            0..=2 => WordStatus::Learning,
            _ => WordStatus::Mastered,
        };
        db.update_word_status(&word.id, word.status);

        let log = ReviewLog::new(word.id.clone(), true, "a greeting");
        db.insert_review_log(&log);

        assert_eq!(word.status, WordStatus::Learning);
        assert_eq!(new_schedule.repetitions, 2);
        assert_eq!(new_schedule.interval, 600);

        // 4. 第三次复习：正确
        let schedule = db.get_schedule(&word.id).unwrap();
        let new_schedule = Sm2Algorithm::correct(&schedule);
        db.update_schedule(&new_schedule);
        word.status = match new_schedule.repetitions {
            0..=2 => WordStatus::Learning,
            _ => WordStatus::Mastered,
        };
        db.update_word_status(&word.id, word.status);

        let log = ReviewLog::new(word.id.clone(), true, "a greeting");
        db.insert_review_log(&log);

        // 第三次后应该变为 Mastered
        assert_eq!(word.status, WordStatus::Mastered);
        assert_eq!(new_schedule.repetitions, 3);
    }

    #[test]
    fn test_word_lifecycle_with_wrong_answers() {
        // 测试复习过程中出现错误答案的情况
        let mut db = MockDatabase::new();

        let definitions = vec![Definition {
            id: "1".to_string(),
            definition: "test definition".to_string(),
            pos: None,
            example: None,
        }];
        let word = create_test_word_with_definitions(&mut db, "test", definitions);

        // 正确、错误、正确、错误、正确的模式
        let answers = vec![true, false, true, false, true, true];

        for (i, is_correct) in answers.iter().enumerate() {
            let schedule = db.get_schedule(&word.id).unwrap();
            let new_schedule = if *is_correct {
                Sm2Algorithm::correct(&schedule)
            } else {
                Sm2Algorithm::incorrect(&schedule)
            };
            db.update_schedule(&new_schedule);

            let status = match new_schedule.repetitions {
                0..=2 => WordStatus::Learning,
                _ => WordStatus::Mastered,
            };
            db.update_word_status(&word.id, status);

            let log = ReviewLog::new(word.id.clone(), *is_correct, "answer");
            db.insert_review_log(&log);

            println!("Step {}: is_correct={}, repetitions={}, status={:?}",
                i + 1, is_correct, new_schedule.repetitions, status);
        }

        // 验证复习日志
        let logs = db.get_review_logs(&word.id);
        assert_eq!(logs.len(), 6);

        let (total, correct) = db.get_review_stats();
        assert_eq!(total, 6);
        assert_eq!(correct, 4); // 4个正确，2个错误
    }

    // ========== 复习流程完整链路测试 ==========

    #[test]
    fn test_review_flow_full_chain() {
        // 模拟完整的复习流程
        let mut db = MockDatabase::new();

        // 1. 添加多个单词
        let _words: Vec<Word> = (0..5)
            .map(|i| {
                let definitions = vec![Definition {
                    id: format!("{}-d1", i),
                    definition: format!("definition {}", i),
                    pos: None,
                    example: None,
                }];
                create_test_word_with_definitions(&mut db, &format!("word{}", i), definitions)
            })
            .collect();

        assert_eq!(db.get_word_count(), 5);

        // 2. 获取下一个待复习单词
        let next = db.get_next_review_word();
        assert!(next.is_some());
        let review_word = next.unwrap();

        // 3. 提交复习答案
        let schedule = db.get_schedule(&review_word.id).unwrap();
        let new_schedule = Sm2Algorithm::correct(&schedule);
        db.update_schedule(&new_schedule);

        let log = ReviewLog::new(review_word.id.clone(), true, "answer");
        db.insert_review_log(&log);

        // 4. 验证统计数据
        let (total, correct) = db.get_review_stats();
        assert_eq!(total, 1);
        assert_eq!(correct, 1);

        // 5. 获取下一个待复习单词（应该不是刚才复习的那个）
        let next2 = db.get_next_review_word();
        assert!(next2.is_some());
    }

    // ========== 跳过和恢复单词测试 ==========

    #[test]
    fn test_skip_and_unskip_word() {
        let mut db = MockDatabase::new();

        let definitions = vec![Definition {
            id: "1".to_string(),
            definition: "test".to_string(),
            pos: None,
            example: None,
        }];
        let word = create_test_word_with_definitions(&mut db, "difficult", definitions);

        // 验证单词可以被获取
        let next = db.get_next_review_word();
        assert!(next.is_some());

        // 跳过单词
        db.update_word_status(&word.id, WordStatus::Skipped);
        db.remove_schedule(&word.id);

        // 跳过后不应该出现在待复习列表
        let next = db.get_next_review_word();
        assert!(next.is_none());

        // 恢复单词
        db.update_word_status(&word.id, WordStatus::Learning);
        let new_schedule = Schedule::new(word.id.clone());
        db.insert_schedule(&new_schedule);

        // 恢复后应该可以获取
        let next = db.get_next_review_word();
        assert!(next.is_some());
        assert_eq!(next.unwrap().word, "difficult");
    }

    // ========== 删除单词测试 ==========

    #[test]
    fn test_delete_word_cascades() {
        let mut db = MockDatabase::new();

        let definitions = vec![Definition {
            id: "1".to_string(),
            definition: "test".to_string(),
            pos: None,
            example: None,
        }];
        let word = create_test_word_with_definitions(&mut db, "todelete", definitions);

        // 添加一些复习日志
        db.insert_review_log(&ReviewLog::new(word.id.clone(), true, "answer1"));
        db.insert_review_log(&ReviewLog::new(word.id.clone(), false, "answer2"));

        assert_eq!(db.get_word_count(), 1);
        let logs = db.get_review_logs(&word.id);
        assert_eq!(logs.len(), 2);

        // 删除单词
        db.delete_word(&word.id);

        assert_eq!(db.get_word_count(), 0);
        let logs = db.get_review_logs(&word.id);
        assert!(logs.is_empty());
        assert!(db.get_schedule(&word.id).is_none());
    }

    // ========== 间隔重复算法完整流程测试 ==========

    #[test]
    fn test_sm2_full_learning_cycle() {
        // 模拟一个完整的 SM-2 学习周期
        let mut schedule = Schedule::new("test-word".to_string());

        // 初始状态
        assert_eq!(schedule.repetitions, 0);
        assert_eq!(schedule.interval, 60);
        assert_eq!(schedule.ease_factor, 2.5);

        // 第一次正确：repetitions = 1, interval = 60s
        schedule = Sm2Algorithm::correct(&schedule);
        assert_eq!(schedule.repetitions, 1);
        assert_eq!(schedule.interval, 60);
        assert!((schedule.ease_factor - 2.6).abs() < 0.001);

        // 第二次正确：repetitions = 2, interval = 600s
        schedule = Sm2Algorithm::correct(&schedule);
        assert_eq!(schedule.repetitions, 2);
        assert_eq!(schedule.interval, 600);
        assert!((schedule.ease_factor - 2.7).abs() < 0.001);

        // 第三次正确：repetitions = 3, interval = 600 * 2.7 = 1620s
        schedule = Sm2Algorithm::correct(&schedule);
        assert_eq!(schedule.repetitions, 3);
        assert_eq!(schedule.interval, 1620);
        assert!((schedule.ease_factor - 2.8).abs() < 0.001);

        // 第四次正确：repetitions = 4, interval = 1620 * 2.8 = 4536s
        schedule = Sm2Algorithm::correct(&schedule);
        assert_eq!(schedule.repetitions, 4);
        assert_eq!(schedule.interval, 4536);

        // 第五次正确：repetitions = 5, interval = 4536 * 2.9 = 13154s
        schedule = Sm2Algorithm::correct(&schedule);
        assert_eq!(schedule.repetitions, 5);
        assert_eq!(schedule.interval, 13154);
    }

    #[test]
    fn test_sm2_incorrect_resets_progress() {
        // 测试错误回答会重置学习进度
        let mut schedule = Schedule::new("test-word".to_string());

        // 学习到一定程度
        // 初始: ease_factor = 2.5
        schedule = Sm2Algorithm::correct(&schedule); // rep=1, ef=2.6
        schedule = Sm2Algorithm::correct(&schedule); // rep=2, ef=2.7
        schedule = Sm2Algorithm::correct(&schedule); // rep=3, ef=2.8
        assert_eq!(schedule.repetitions, 3);
        assert_eq!(schedule.interval, 1620);
        assert!((schedule.ease_factor - 2.8).abs() < 0.001);

        // 错误回答：重置 repetitions 和 interval，ease_factor - 0.2
        schedule = Sm2Algorithm::incorrect(&schedule);

        // 应该重置
        assert_eq!(schedule.repetitions, 0);
        assert_eq!(schedule.interval, 60);
        // ease_factor = 2.8 - 0.2 = 2.6
        assert!((schedule.ease_factor - 2.6).abs() < 0.001);
    }

    #[test]
    fn test_sm2_multiple_incorrect_in_row() {
        // 测试连续错误回答
        let mut schedule = Schedule::new("test-word".to_string());

        // 初始 ease_factor
        let initial_ef = schedule.ease_factor;

        // 连续三次错误
        schedule = Sm2Algorithm::incorrect(&schedule);
        let ef1 = schedule.ease_factor;

        schedule = Sm2Algorithm::incorrect(&schedule);
        let ef2 = schedule.ease_factor;

        schedule = Sm2Algorithm::incorrect(&schedule);
        let ef3 = schedule.ease_factor;

        // ease_factor 应该逐渐降低
        assert!(ef1 < initial_ef);
        assert!(ef2 < ef1);
        assert!(ef3 < ef2);

        // 但不应该低于最小值 1.3
        assert!(ef3 >= 1.3);

        // 验证间隔也重置了
        assert_eq!(schedule.repetitions, 0);
        assert_eq!(schedule.interval, 60);
    }

    // ========== 批量操作测试 ==========

    #[test]
    fn test_batch_word_operations() {
        let mut db = MockDatabase::new();

        // 批量添加单词
        for i in 0..10 {
            let definitions = vec![Definition {
                id: format!("{}-d1", i),
                definition: format!("definition {}", i),
                pos: None,
                example: None,
            }];
            create_test_word_with_definitions(&mut db, &format!("word{}", i), definitions);
        }

        assert_eq!(db.get_word_count(), 10);

        // 批量复习
        for _ in 0..5 {
            if let Some(word) = db.get_next_review_word() {
                let schedule = db.get_schedule(&word.id).unwrap();
                let new_schedule = Sm2Algorithm::correct(&schedule);
                db.update_schedule(&new_schedule);
                db.insert_review_log(&ReviewLog::new(word.id, true, "answer"));
            }
        }

        // 验证统计
        let (total, correct) = db.get_review_stats();
        assert_eq!(total, 5);
        assert_eq!(correct, 5);
    }

    // ========== 状态转换完整性测试 ==========

    #[test]
    fn test_all_status_transitions() {
        let mut db = MockDatabase::new();

        let definitions = vec![Definition {
            id: "1".to_string(),
            definition: "test".to_string(),
            pos: None,
            example: None,
        }];
        let word = create_test_word_with_definitions(&mut db, "status-test", definitions);

        // 测试 New -> Learning
        assert_eq!(word.status, WordStatus::New);
        db.update_word_status(&word.id, WordStatus::Learning);
        assert_eq!(db.get_word(&word.id).unwrap().status, WordStatus::Learning);

        // 测试 Learning -> Mastered
        db.update_word_status(&word.id, WordStatus::Mastered);
        assert_eq!(db.get_word(&word.id).unwrap().status, WordStatus::Mastered);

        // 测试 Mastered -> Learning (重新学习)
        db.update_word_status(&word.id, WordStatus::Learning);
        assert_eq!(db.get_word(&word.id).unwrap().status, WordStatus::Learning);

        // 测试 Learning -> Skipped
        db.update_word_status(&word.id, WordStatus::Skipped);
        assert_eq!(db.get_word(&word.id).unwrap().status, WordStatus::Skipped);

        // 测试 Skipped -> Learning
        db.update_word_status(&word.id, WordStatus::Learning);
        assert_eq!(db.get_word(&word.id).unwrap().status, WordStatus::Learning);
    }

    // ========== 时间顺序测试 ==========

    #[test]
    fn test_schedule_time_ordering() {
        let mut db = MockDatabase::new();

        // 创建单词，每个间隔不同
        for i in 0..5 {
            let definitions = vec![Definition {
                id: format!("{}-d1", i),
                definition: format!("definition {}", i),
                pos: None,
                example: None,
            }];
            let word = create_test_word_with_definitions(&mut db, &format!("word{}", i), definitions);

            // 修改复习时间
            let mut schedule = db.get_schedule(&word.id).unwrap();
            schedule.next_review = (i as i64) * 1000; // word0: 0ms, word1: 1000ms, etc.
            db.update_schedule(&schedule);
        }

        // 验证 get_next_review_word 返回最近应该复习的单词
        let next = db.get_next_review_word();
        assert!(next.is_some());
        assert_eq!(next.unwrap().word, "word0"); // 最近应该复习
    }

    // ========== 边界条件测试 ==========

    #[test]
    fn test_empty_database_operations() {
        let db = MockDatabase::new();

        // 空数据库操作
        assert_eq!(db.get_word_count(), 0);
        assert!(db.get_next_review_word().is_none());
        let (total, correct) = db.get_review_stats();
        assert_eq!(total, 0);
        assert_eq!(correct, 0);
    }

    #[test]
    fn test_nonexistent_word_operations() {
        let db = MockDatabase::new();

        // 操作不存在的单词
        assert!(db.get_word("nonexistent").is_none());
        assert!(db.get_schedule("nonexistent").is_none());

        let logs = db.get_review_logs("nonexistent");
        assert!(logs.is_empty());
    }
}
