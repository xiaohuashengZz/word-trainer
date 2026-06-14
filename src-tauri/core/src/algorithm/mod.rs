//! 间隔重复算法 - SM-2 简化版

use crate::domain::Schedule;

/// Ease factor 最小值
const MIN_EASE_FACTOR: f64 = 1.3;
/// 浮点精度容差
const FP_TOLERANCE: f64 = 0.001;

pub struct Sm2Algorithm;

impl Sm2Algorithm {
    /// 正确回答时的调度计算
    pub fn correct(schedule: &Schedule) -> Schedule {
        let mut new_schedule = schedule.clone();
        let now = chrono::Utc::now().timestamp_millis();
        new_schedule.repetitions += 1;

        let new_interval = match new_schedule.repetitions {
            1 => 60,
            2 => 600,
            _ => (schedule.interval as f64 * new_schedule.ease_factor) as i64,
        };

        // 计算新的 ease factor：如果当前值接近最小值，保持最小值
        let raw_ef = new_schedule.ease_factor + 0.1;
        new_schedule.ease_factor = if (new_schedule.ease_factor - MIN_EASE_FACTOR).abs() < FP_TOLERANCE {
            MIN_EASE_FACTOR
        } else if raw_ef < MIN_EASE_FACTOR {
            MIN_EASE_FACTOR
        } else {
            raw_ef
        };
        new_schedule.interval = new_interval;
        new_schedule.last_review = Some(now);
        new_schedule.next_review = now + (new_interval * 1000);

        new_schedule
    }

    /// 错误回答时的调度计算
    pub fn incorrect(schedule: &Schedule) -> Schedule {
        let mut new_schedule = schedule.clone();
        let now = chrono::Utc::now().timestamp_millis();

        new_schedule.repetitions = 0;
        new_schedule.interval = 60;
        let raw_ef = new_schedule.ease_factor - 0.2;
        new_schedule.ease_factor = if raw_ef < MIN_EASE_FACTOR { MIN_EASE_FACTOR } else { raw_ef };
        new_schedule.last_review = Some(now);
        new_schedule.next_review = now + 60_000;

        new_schedule
    }

    /// 计算下一个复习时间点（以毫秒计）
    pub fn calculate_next_review_time(interval_ms: i64) -> i64 {
        chrono::Utc::now().timestamp_millis() + interval_ms
    }

    /// 根据重复次数计算新的间隔（秒）
    pub fn calculate_interval(repetitions: i32, ease_factor: f64, current_interval: i64) -> i64 {
        match repetitions {
            1 => 60,
            2 => 600,
            _ => (current_interval as f64 * ease_factor) as i64,
        }
    }

    /// 计算新的 ease factor
    pub fn calculate_ease_factor(current: f64, is_correct: bool) -> f64 {
        // 如果当前值接近最小值，保持最小值（避免浮点精度问题）
        if (current - MIN_EASE_FACTOR).abs() < FP_TOLERANCE {
            return MIN_EASE_FACTOR;
        }
        let raw = if is_correct { current + 0.1 } else { current - 0.2 };
        if raw < MIN_EASE_FACTOR { MIN_EASE_FACTOR } else { raw }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_schedule() -> Schedule {
        Schedule {
            word_id: "test-id".to_string(),
            interval: 60,
            ease_factor: 2.5,
            repetitions: 0,
            next_review: 0,
            last_review: None,
        }
    }

    fn create_schedule(word_id: &str, interval: i64, ease_factor: f64, repetitions: i32) -> Schedule {
        Schedule {
            word_id: word_id.to_string(),
            interval,
            ease_factor,
            repetitions,
            next_review: 0,
            last_review: None,
        }
    }

    // ========== 正确回答测试 ==========
    #[test]
    fn test_correct_first_repetition() {
        let schedule = create_test_schedule();
        let result = Sm2Algorithm::correct(&schedule);

        assert_eq!(result.repetitions, 1);
        assert_eq!(result.interval, 60); // 第一次: 60秒
        assert_eq!(result.ease_factor, 2.6); // 2.5 + 0.1
        assert!(result.last_review.is_some());
    }

    #[test]
    fn test_correct_second_repetition() {
        let mut schedule = create_test_schedule();
        schedule.repetitions = 1;
        schedule.interval = 60;

        let result = Sm2Algorithm::correct(&schedule);

        assert_eq!(result.repetitions, 2);
        assert_eq!(result.interval, 600); // 第二次: 600秒 = 10分钟
    }

    #[test]
    fn test_correct_third_repetition() {
        let mut schedule = create_test_schedule();
        schedule.repetitions = 2;
        schedule.interval = 600;
        schedule.ease_factor = 2.6;

        let result = Sm2Algorithm::correct(&schedule);

        assert_eq!(result.repetitions, 3);
        // 第三次: 600 * 2.6 = 1560秒
        assert_eq!(result.interval, 1560);
    }

    #[test]
    fn test_correct_fourth_repetition() {
        let schedule = create_schedule("test", 1560, 2.7, 3);
        let result = Sm2Algorithm::correct(&schedule);

        assert_eq!(result.repetitions, 4);
        // 第四次: 1560 * 2.7 = 4212秒 ≈ 70分钟
        assert_eq!(result.interval, 4212);
    }

    #[test]
    fn test_correct_increases_ease_factor() {
        let schedule = create_test_schedule();
        let result = Sm2Algorithm::correct(&schedule);

        // 每次正确回答 ease_factor + 0.1
        assert_eq!(result.ease_factor, 2.6);
    }

    #[test]
    fn test_correct_updates_last_review() {
        let schedule = create_test_schedule();
        let before = chrono::Utc::now().timestamp_millis();
        let result = Sm2Algorithm::correct(&schedule);
        let after = chrono::Utc::now().timestamp_millis();

        assert!(result.last_review.is_some());
        let last_review = result.last_review.unwrap();
        assert!(last_review >= before && last_review <= after);
    }

    // ========== 错误回答测试 ==========
    #[test]
    fn test_incorrect_resets_repetitions() {
        let mut schedule = create_test_schedule();
        schedule.repetitions = 5;
        schedule.interval = 1000;
        schedule.ease_factor = 3.0;

        let result = Sm2Algorithm::incorrect(&schedule);

        assert_eq!(result.repetitions, 0);
        assert_eq!(result.interval, 60);
        assert_eq!(result.ease_factor, 2.8); // 3.0 - 0.2
    }

    #[test]
    fn test_incorrect_minimum_ease_factor() {
        let mut schedule = create_test_schedule();
        schedule.ease_factor = 1.4;

        let result = Sm2Algorithm::incorrect(&schedule);

        // ease_factor 最低为 1.3
        assert_eq!(result.ease_factor, 1.3);
    }

    #[test]
    fn test_incorrect_resets_interval_to_60() {
        let schedule = create_schedule("test", 10000, 2.5, 10);
        let result = Sm2Algorithm::incorrect(&schedule);

        assert_eq!(result.interval, 60);
    }

    #[test]
    fn test_incorrect_decreases_ease_factor() {
        let mut schedule = create_test_schedule();
        schedule.ease_factor = 3.0;

        let result = Sm2Algorithm::incorrect(&schedule);

        assert_eq!(result.ease_factor, 2.8); // 3.0 - 0.2
    }

    // ========== Ease Factor 边界测试 ==========
    #[test]
    fn test_correct_ease_factor_minimum() {
        let mut schedule = create_test_schedule();
        schedule.ease_factor = 1.3;
        schedule.repetitions = 2;

        let result = Sm2Algorithm::correct(&schedule);

        // ease_factor 最低为 1.3（浮点数精度测试）
        assert!((result.ease_factor - 1.3).abs() < 0.0001,
            "ease_factor = {}, expected ~1.3", result.ease_factor);
    }

    #[test]
    fn test_ease_factor_at_minimum_boundary() {
        // 测试 ease_factor = 1.31 时的行为
        let schedule = create_schedule("test", 60, 1.31, 1);
        let result = Sm2Algorithm::correct(&schedule);

        // 1.31 + 0.1 = 1.41 > 1.3，应该正常增加
        assert!((result.ease_factor - 1.41).abs() < 0.001);
    }

    #[test]
    fn test_ease_factor_at_maximum_reasonable_value() {
        let schedule = create_schedule("test", 60, 5.0, 1);
        let result = Sm2Algorithm::correct(&schedule);

        assert_eq!(result.ease_factor, 5.1);
    }

    // ========== 时间计算测试 ==========
    #[test]
    fn test_next_review_is_future() {
        let schedule = create_test_schedule();
        let now = chrono::Utc::now().timestamp_millis();

        let result = Sm2Algorithm::correct(&schedule);

        assert!(result.next_review > now);
    }

    #[test]
    fn test_next_review_calculation_correct() {
        let mut schedule = create_test_schedule();
        schedule.repetitions = 1; // 第二次复习

        let before = chrono::Utc::now().timestamp_millis();
        let result = Sm2Algorithm::correct(&schedule);
        let after = chrono::Utc::now().timestamp_millis();

        // 间隔 600 秒 = 600000 毫秒
        assert!(result.next_review >= before + 600000 - 1000); // 允许1秒误差
        assert!(result.next_review <= after + 600000 + 1000);
    }

    #[test]
    fn test_incorrect_next_review_is_60_seconds() {
        let schedule = create_test_schedule();
        let before = chrono::Utc::now().timestamp_millis();
        let result = Sm2Algorithm::incorrect(&schedule);
        let after = chrono::Utc::now().timestamp_millis();

        // 错误后 60 秒 = 60000 毫秒
        assert!(result.next_review >= before + 60000 - 1000);
        assert!(result.next_review <= after + 60000 + 1000);
    }

    // ========== 间隔计算测试 ==========
    #[test]
    fn test_calculate_interval_first() {
        assert_eq!(Sm2Algorithm::calculate_interval(1, 2.5, 60), 60);
    }

    #[test]
    fn test_calculate_interval_second() {
        assert_eq!(Sm2Algorithm::calculate_interval(2, 2.5, 60), 600);
    }

    #[test]
    fn test_calculate_interval_third_and_beyond() {
        assert_eq!(Sm2Algorithm::calculate_interval(3, 2.5, 600), 1500);
        assert_eq!(Sm2Algorithm::calculate_interval(4, 2.5, 1500), 3750);
        assert_eq!(Sm2Algorithm::calculate_interval(5, 2.5, 3750), 9375);
    }

    #[test]
    fn test_calculate_interval_with_different_ease_factors() {
        // 不同 ease_factor 导致不同间隔
        assert_eq!(Sm2Algorithm::calculate_interval(3, 1.5, 100), 150);
        assert_eq!(Sm2Algorithm::calculate_interval(3, 2.0, 100), 200);
        assert_eq!(Sm2Algorithm::calculate_interval(3, 3.0, 100), 300);
    }

    // ========== Ease Factor 计算测试 ==========
    #[test]
    fn test_calculate_ease_factor_correct() {
        assert!((Sm2Algorithm::calculate_ease_factor(2.5, true) - 2.6).abs() < 0.0001);
        // 最低为 1.3：当 1.3 + 0.1 = 1.40000... 时，应该被 clamp 回 1.3
        let ef = Sm2Algorithm::calculate_ease_factor(1.3, true);
        assert!((ef - 1.3).abs() < 0.0001, "ease_factor = {}, expected ~1.3", ef);
    }

    #[test]
    fn test_calculate_ease_factor_incorrect() {
        assert!((Sm2Algorithm::calculate_ease_factor(2.5, false) - 2.3).abs() < 0.0001);
        // 最低为 1.3
        let ef = Sm2Algorithm::calculate_ease_factor(1.3, false);
        assert!((ef - 1.3).abs() < 0.0001, "ease_factor = {}, expected ~1.3", ef);
    }

    #[test]
    fn test_calculate_ease_factor_consecutive_correct() {
        let mut ef = 2.5;
        for _ in 0..10 {
            ef = Sm2Algorithm::calculate_ease_factor(ef, true);
        }
        // 2.5 + 0.1 * 10 = 3.5
        assert!((ef - 3.5).abs() < 0.001);
    }

    #[test]
    fn test_calculate_ease_factor_consecutive_incorrect() {
        let mut ef = 2.5;
        for _ in 0..5 {
            ef = Sm2Algorithm::calculate_ease_factor(ef, false);
        }
        // 2.5 - 0.2 * 5 = 1.5
        assert!((ef - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_calculate_ease_factor_mixed() {
        let mut ef = 2.5;
        // 正确、错误、正确、错误、正确
        ef = Sm2Algorithm::calculate_ease_factor(ef, true);  // 2.6
        ef = Sm2Algorithm::calculate_ease_factor(ef, false); // 2.4
        ef = Sm2Algorithm::calculate_ease_factor(ef, true);  // 2.5
        ef = Sm2Algorithm::calculate_ease_factor(ef, false); // 2.3
        ef = Sm2Algorithm::calculate_ease_factor(ef, true);  // 2.4

        assert!((ef - 2.4).abs() < 0.001);
    }

    // ========== Schedule 不变性测试 ==========
    #[test]
    fn test_correct_does_not_modify_original() {
        let schedule = create_test_schedule();
        let original_id = schedule.word_id.clone();
        let _result = Sm2Algorithm::correct(&schedule);

        assert_eq!(schedule.word_id, original_id);
        assert_eq!(schedule.repetitions, 0); // 未被修改
    }

    #[test]
    fn test_incorrect_does_not_modify_original() {
        let schedule = create_test_schedule();
        let original_id = schedule.word_id.clone();
        let _result = Sm2Algorithm::incorrect(&schedule);

        assert_eq!(schedule.word_id, original_id);
        assert_eq!(schedule.ease_factor, 2.5); // 未被修改
    }

    // ========== 边界值测试 ==========
    #[test]
    fn test_very_small_interval() {
        let schedule = create_schedule("test", 1, 2.5, 2);
        let result = Sm2Algorithm::correct(&schedule);

        // 1 * 2.5 = 2.5 -> 2 (取整)
        assert_eq!(result.interval, 2);
    }

    #[test]
    fn test_very_large_ease_factor() {
        let schedule = create_schedule("test", 60, 10.0, 2);
        let result = Sm2Algorithm::correct(&schedule);

        // 第三次复习：interval = 60 * 10.0 = 600
        // ease_factor = 10.0 + 0.1 = 10.1
        assert_eq!(result.interval, 600);
        assert_eq!(result.ease_factor, 10.1);
    }

    #[test]
    fn test_zero_repetitions_after_incorrect() {
        let schedule = create_schedule("test", 600, 2.5, 10);
        let result = Sm2Algorithm::incorrect(&schedule);

        assert_eq!(result.repetitions, 0);
        assert_eq!(result.interval, 60); // 重置为 60 秒
    }
}
