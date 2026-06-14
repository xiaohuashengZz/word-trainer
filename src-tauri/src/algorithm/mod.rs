//! 间隔重复算法 - SM-2 简化版

use crate::domain::Schedule;

pub struct Sm2Algorithm;

impl Sm2Algorithm {
    pub fn correct(schedule: &Schedule) -> Schedule {
        let mut new_schedule = schedule.clone();
        let now = chrono::Utc::now().timestamp_millis();
        new_schedule.repetitions += 1;

        let new_interval = match new_schedule.repetitions {
            1 => 60,
            2 => 600,
            _ => (schedule.interval as f64 * new_schedule.ease_factor) as i64,
        };

        new_schedule.ease_factor = (new_schedule.ease_factor + 0.1).max(1.3);
        new_schedule.interval = new_interval;
        new_schedule.last_review = Some(now);
        new_schedule.next_review = now + (new_interval * 1000);

        new_schedule
    }

    pub fn incorrect(schedule: &Schedule) -> Schedule {
        let mut new_schedule = schedule.clone();
        let now = chrono::Utc::now().timestamp_millis();

        new_schedule.repetitions = 0;
        new_schedule.interval = 60;
        new_schedule.ease_factor = (new_schedule.ease_factor - 0.2).max(1.3);
        new_schedule.last_review = Some(now);
        new_schedule.next_review = now + 60_000;

        new_schedule
    }
}