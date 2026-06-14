//! 领域层 - 核心业务实体

mod word;
mod schedule;
mod review_log;

pub use word::{Word, Definition, WordStatus};
pub use schedule::Schedule;
pub use review_log::ReviewLog;

#[cfg(test)]
mod word_test;