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

// 单词本下载命令
#[tauri::command]
pub async fn download_word_book(
    db: State<'_, Arc<Database>>,
    book_id: String,
    book_name: String,
) -> Result<i32, String> {
    log::info!("下载词库: {} ({})", book_name, book_id);

    // 获取内置单词数据
    let words = get_word_book_data(&book_id);
    let mut added_count = 0;

    for (word_text, phonetic, definition) in words {
        // 检查单词是否已存在
        if db.get_word_count_by_text(word_text).map_err(|e| e.to_string())? > 0 {
            continue;
        }

        let mut word = Word::new(word_text.to_string());
        word.phonetic = Some(phonetic.to_string());
        word.definitions = vec![Definition {
            id: uuid::Uuid::new_v4().to_string(),
            pos: None,
            definition: definition.to_string(),
            example: None,
        }];

        db.insert_word(&word).map_err(|e| e.to_string())?;
        if let Some(def) = word.definitions.first() {
            db.insert_definition(&word.id, def).map_err(|e| e.to_string())?;
        }
        let schedule = Schedule::new(word.id.clone());
        db.insert_schedule(&schedule).map_err(|e| e.to_string())?;
        added_count += 1;
    }

    log::info!("成功添加 {} 个单词", added_count);
    Ok(added_count)
}

// 内置单词数据
fn get_word_book_data(book_id: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    match book_id {
        "cet4" => vec![
            ("abandon", "/əˈbændən/", "放弃；遗弃"),
            ("ability", "/əˈbɪləti/", "能力"),
            ("able", "/ˈeɪbl/", "能够的；有能力的"),
            ("about", "/əˈbaʊt/", "关于；大约"),
            ("above", "/əˈbʌv/", "在...上面"),
            ("abroad", "/əˈbrɔːd/", "在国外；到国外"),
            ("absence", "/ˈæbsəns/", "缺席；缺乏"),
            ("absolute", "/ˈæbsəluːt/", "绝对的；完全的"),
            ("absorb", "/əbˈzɔːrb/", "吸收；理解"),
            ("abstract", "/ˈæbstrækt/", "抽象的；摘要"),
            ("abundant", "/əˈbʌndənt/", "丰富的；大量的"),
            ("academic", "/ˌækəˈdemɪk/", "学术的；学院的"),
            ("accept", "/əkˈsept/", "接受；承认"),
            ("access", "/ˈækses/", "进入；访问；通路"),
            ("accident", "/ˈæksɪdənt/", "事故；意外"),
            ("accompany", "/əˈkʌmpəni/", "陪伴；伴随"),
            ("accomplish", "/əˈkʌmplɪʃ/", "完成；实现"),
            ("according", "/əˈkɔːrdɪŋ/", "根据；按照"),
            ("account", "/əˈkaʊnt/", "账户；叙述"),
            ("accurate", "/ˈækjərət/", "准确的；精确的"),
            ("achieve", "/əˈtʃiːv/", "实现；达到"),
            ("acknowledge", "/əkˈnɒlɪdʒ/", "承认；确认"),
            ("acquire", "/əˈkwaɪər/", "获得；学到"),
            ("across", "/əˈkrɔːs/", "穿过；在对面"),
            ("active", "/ˈæktɪv/", "积极的；活跃的"),
            ("activity", "/ækˈtɪvəti/", "活动；行动"),
            ("actual", "/ˈæktʃuəl/", "实际的；真实的"),
            ("adapt", "/əˈdæpt/", "适应；改编"),
            ("address", "/əˈdres/", "地址；演说"),
            ("adequate", "/ˈædɪkwət/", "充足的；适当的"),
        ],
        "cet6" => vec![
            ("abandon", "/əˈbændən/", "放弃；遗弃"),
            ("ability", "/əˈbɪləti/", "能力"),
            ("abnormal", "/æbˈnɔːrməl/", "反常的；异常的"),
            ("aboard", "/əˈbɔːrd/", "在船（飞机、车）上"),
            ("abolish", "/əˈbɒlɪʃ/", "废除；取消"),
            ("abortion", "/əˈbɔːrʃən/", "流产；堕胎"),
            ("absence", "/ˈæbsəns/", "缺席；缺乏"),
            ("absolute", "/ˈæbsəluːt/", "绝对的；完全的"),
            ("absorb", "/əbˈzɔːrb/", "吸收；理解"),
            ("abstract", "/ˈæbstrækt/", "抽象的；摘要"),
            ("abundant", "/əˈbʌndənt/", "丰富的；大量的"),
            ("academic", "/ˌækəˈdemɪk/", "学术的；学院的"),
            ("accelerate", "/əkˈseləreɪt/", "加速；加快"),
            ("accept", "/əkˈsept/", "接受；承认"),
            ("access", "/ˈækses/", "进入；访问"),
            ("accident", "/ˈæksɪdənt/", "事故；意外"),
            ("accommodate", "/əˈkɒmədeɪt/", "容纳；使适应"),
            ("accompany", "/əˈkʌmpəni/", "陪伴；伴随"),
            ("accomplish", "/əˈkʌmplɪʃ/", "完成；实现"),
            ("accord", "/əˈkɔːrd/", "一致；给予"),
            ("accumulate", "/əˈkjuːmjəleɪt/", "积累；堆积"),
            ("accurate", "/ˈækjərət/", "准确的；精确的"),
            ("achieve", "/əˈtʃiːv/", "实现；达到"),
            ("acknowledge", "/əkˈnɒlɪdʒ/", "承认；确认"),
            ("acquire", "/əˈkwaɪər/", "获得；学到"),
            ("adapt", "/əˈdæpt/", "适应；改编"),
            ("adequate", "/ˈædɪkwət/", "充足的"),
            ("adjust", "/əˈdʒʌst/", "调整；适应"),
        ],
        "primary1" => vec![
            ("hello", "/həˈloʊ/", "你好"),
            ("world", "/wɜːrld/", "世界"),
            ("good", "/ɡʊd/", "好的"),
            ("morning", "/ˈmɔːrnɪŋ/", "早上"),
            ("afternoon", "/ˌæftərˈnuːn/", "下午"),
            ("evening", "/ˈiːvnɪŋ/", "晚上"),
            ("night", "/naɪt/", "夜晚"),
            ("cat", "/kæt/", "猫"),
            ("dog", "/dɔːɡ/", "狗"),
            ("bird", "/bɜːrd/", "鸟"),
            ("fish", "/fɪʃ/", "鱼"),
            ("one", "/wʌn/", "一"),
            ("two", "/tuː/", "二"),
            ("three", "/θriː/", "三"),
            ("red", "/red/", "红色"),
            ("blue", "/bluː/", "蓝色"),
            ("yellow", "/ˈjeloʊ/", "黄色"),
            ("green", "/ɡriːn/", "绿色"),
            ("book", "/bʊk/", "书"),
            ("pen", "/pen/", "钢笔"),
        ],
        _ => vec![
            ("hello", "/həˈloʊ/", "你好"),
            ("world", "/wɜːrld/", "世界"),
            ("study", "/ˈstʌdi/", "学习"),
            ("english", "/ˈɪŋɡlɪʃ/", "英语"),
            ("word", "/wɜːrd/", "单词"),
            ("learn", "/lɜːrn/", "学习"),
            ("book", "/bʊk/", "书"),
            ("read", "/riːd/", "阅读"),
            ("write", "/raɪt/", "写"),
            ("speak", "/spiːk/", "说"),
        ],
    }.to_vec()
}
