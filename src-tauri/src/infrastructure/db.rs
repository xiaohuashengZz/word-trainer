//! SQLite 数据库操作

use rusqlite::{Connection, Result as SqliteResult, params};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::domain::{Definition, ReviewLog, Schedule, Word, WordStatus};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(data_dir: PathBuf) -> SqliteResult<Self> {
        std::fs::create_dir_all(&data_dir).ok();
        let db_path = data_dir.join("word-trainer.db");
        log::info!("Opening database at: {:?}", db_path);
        let conn = Connection::open(&db_path)?;
        let db = Self { conn: Mutex::new(conn) };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS words (
                id TEXT PRIMARY KEY, word TEXT NOT NULL, phonetic TEXT,
                phonetic_audio_url TEXT, status TEXT DEFAULT 'new',
                created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS definitions (
                id TEXT PRIMARY KEY, word_id TEXT NOT NULL, pos TEXT,
                definition TEXT NOT NULL, example TEXT, sort_order INTEGER DEFAULT 0,
                FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS schedules (
                word_id TEXT PRIMARY KEY, interval INTEGER DEFAULT 60,
                ease_factor REAL DEFAULT 2.5, repetitions INTEGER DEFAULT 0,
                next_review INTEGER NOT NULL, last_review INTEGER,
                FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS review_logs (
                id TEXT PRIMARY KEY, word_id TEXT NOT NULL, is_correct INTEGER NOT NULL,
                user_answer TEXT, reviewed_at INTEGER NOT NULL,
                FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS skip_logs (
                id TEXT PRIMARY KEY, word_id TEXT NOT NULL, skipped_at INTEGER NOT NULL, reason TEXT,
                FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY, value TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_definitions_word_id ON definitions(word_id);
            CREATE INDEX IF NOT EXISTS idx_schedules_next_review ON schedules(next_review);
            CREATE INDEX IF NOT EXISTS idx_words_status ON words(status);
        "#)?;
        log::info!("Database tables initialized");
        Ok(())
    }

    pub fn insert_word(&self, word: &Word) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO words (id, word, phonetic, phonetic_audio_url, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![word.id, word.word, word.phonetic, word.phonetic_audio_url,
                   word.status.to_str(), word.created_at, word.updated_at],
        )?;
        Ok(())
    }

    pub fn insert_definition(&self, word_id: &str, def: &Definition) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO definitions (id, word_id, pos, definition, example, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![def.id, word_id, def.pos, def.definition, def.example, 0],
        )?;
        Ok(())
    }

    pub fn insert_schedule(&self, schedule: &Schedule) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO schedules (word_id, interval, ease_factor, repetitions, next_review, last_review)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![schedule.word_id, schedule.interval, schedule.ease_factor,
                   schedule.repetitions, schedule.next_review, schedule.last_review],
        )?;
        Ok(())
    }

    pub fn get_next_review_word(&self) -> SqliteResult<Option<Word>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        let mut stmt = conn.prepare(
            "SELECT w.id, w.word, w.phonetic, w.phonetic_audio_url, w.status, w.created_at, w.updated_at
             FROM words w INNER JOIN schedules s ON w.id = s.word_id
             WHERE w.status != 'skipped' ORDER BY s.next_review ASC LIMIT 1"
        )?;
        let mut rows = stmt.query(params![now])?;
        if let Some(row) = rows.next()? {
            let word = Word {
                id: row.get(0)?, word: row.get(1)?, phonetic: row.get(2)?,
                phonetic_audio_url: row.get(3)?, definitions: Vec::new(),
                status: WordStatus::from_str(&row.get::<_, String>(4)?),
                created_at: row.get(5)?, updated_at: row.get(6)?,
            };
            drop(rows);
            drop(stmt);
            let defs = self.get_definitions(&word.id)?;
            Ok(Some(Word { definitions: defs, ..word }))
        } else {
            Ok(None)
        }
    }

    pub fn get_word_by_id(&self, word_id: &str) -> SqliteResult<Option<Word>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, word, phonetic, phonetic_audio_url, status, created_at, updated_at FROM words WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![word_id])?;
        if let Some(row) = rows.next()? {
            let word = Word {
                id: row.get(0)?, word: row.get(1)?, phonetic: row.get(2)?,
                phonetic_audio_url: row.get(3)?, definitions: Vec::new(),
                status: WordStatus::from_str(&row.get::<_, String>(4)?),
                created_at: row.get(5)?, updated_at: row.get(6)?,
            };
            drop(rows);
            drop(stmt);
            let defs = self.get_definitions(&word.id)?;
            Ok(Some(Word { definitions: defs, ..word }))
        } else {
            Ok(None)
        }
    }

    fn get_definitions(&self, word_id: &str) -> SqliteResult<Vec<Definition>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, pos, definition, example FROM definitions WHERE word_id = ?1 ORDER BY sort_order")?;
        let defs = stmt.query_map(params![word_id], |row| {
            Ok(Definition {
                id: row.get(0)?, pos: row.get(1)?, definition: row.get(2)?, example: row.get(3)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(defs)
    }

    pub fn update_word_status(&self, word_id: &str, status: WordStatus) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute("UPDATE words SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    params![status.to_str(), now, word_id])?;
        Ok(())
    }

    pub fn update_schedule(&self, schedule: &Schedule) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE schedules SET interval = ?1, ease_factor = ?2, repetitions = ?3, next_review = ?4, last_review = ?5 WHERE word_id = ?6",
            params![schedule.interval, schedule.ease_factor, schedule.repetitions,
                   schedule.next_review, schedule.last_review, schedule.word_id],
        )?;
        Ok(())
    }

    pub fn insert_review_log(&self, log: &ReviewLog) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO review_logs (id, word_id, is_correct, user_answer, reviewed_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![log.id, log.word_id, log.is_correct as i32, log.user_answer, log.reviewed_at],
        )?;
        Ok(())
    }

    pub fn get_schedule(&self, word_id: &str) -> SqliteResult<Schedule> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT word_id, interval, ease_factor, repetitions, next_review, last_review FROM schedules WHERE word_id = ?1"
        )?;
        let schedule = stmt.query_row(params![word_id], |row| {
            Ok(Schedule {
                word_id: row.get(0)?, interval: row.get(1)?, ease_factor: row.get(2)?,
                repetitions: row.get(3)?, next_review: row.get(4)?, last_review: row.get(5)?,
            })
        })?;
        Ok(schedule)
    }

    pub fn remove_from_schedule(&self, word_id: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM schedules WHERE word_id = ?1", params![word_id])?;
        Ok(())
    }

    pub fn insert_skip_log(&self, word_id: &str, reason: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute("INSERT INTO skip_logs (id, word_id, skipped_at, reason) VALUES (?1, ?2, ?3, ?4)",
                    params![id, word_id, now, reason])?;
        Ok(())
    }

    pub fn get_word_count(&self) -> SqliteResult<i32> {
        let conn = self.conn.lock().unwrap();
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM words", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_word_count_by_text(&self, word_text: &str) -> SqliteResult<i32> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM words WHERE LOWER(word) = LOWER(?1)")?;
        let count: i32 = stmt.query_row(params![word_text], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_word_counts_by_status(&self) -> SqliteResult<(i32, i32, i32, i32)> {
        let conn = self.conn.lock().unwrap();
        let mut new_count = 0i32; let mut learning_count = 0i32;
        let mut mastered_count = 0i32; let mut skipped_count = 0i32;
        let mut stmt = conn.prepare("SELECT status, COUNT(*) FROM words GROUP BY status")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)))?;
        for row in rows.flatten() {
            match row.0.as_str() {
                "new" => new_count = row.1, "learning" => learning_count = row.1,
                "mastered" => mastered_count = row.1, "skipped" => skipped_count = row.1, _ => {}
            }
        }
        Ok((new_count, learning_count, mastered_count, skipped_count))
    }

    pub fn get_review_stats(&self) -> SqliteResult<(i32, i32, i32)> {
        let conn = self.conn.lock().unwrap();
        let total: i32 = conn.query_row("SELECT COUNT(*) FROM review_logs", [], |row| row.get(0))?;
        let correct: i32 = conn.query_row("SELECT COUNT(*) FROM review_logs WHERE is_correct = 1", [], |row| row.get(0))?;
        Ok((total, correct, total - correct))
    }

    pub fn delete_word(&self, word_id: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM definitions WHERE word_id = ?1", params![word_id])?;
        conn.execute("DELETE FROM schedules WHERE word_id = ?1", params![word_id])?;
        conn.execute("DELETE FROM review_logs WHERE word_id = ?1", params![word_id])?;
        conn.execute("DELETE FROM skip_logs WHERE word_id = ?1", params![word_id])?;
        conn.execute("DELETE FROM words WHERE id = ?1", params![word_id])?;
        Ok(())
    }

    pub fn list_words(&self, status: Option<&str>, offset: i32, limit: i32) -> SqliteResult<Vec<Word>> {
        let conn = self.conn.lock().unwrap();
        let query = match status {
            Some(s) => format!("SELECT id, word, phonetic, phonetic_audio_url, status, created_at, updated_at FROM words WHERE status = '{}' ORDER BY updated_at DESC LIMIT {} OFFSET {}",
                              s, limit, offset),
            None => format!("SELECT id, word, phonetic, phonetic_audio_url, status, created_at, updated_at FROM words ORDER BY updated_at DESC LIMIT {} OFFSET {}",
                          limit, offset),
        };
        let mut stmt = conn.prepare(&query)?;
        let words: Vec<Word> = stmt.query_map([], |row| {
            Ok(Word {
                id: row.get(0)?, word: row.get(1)?, phonetic: row.get(2)?,
                phonetic_audio_url: row.get(3)?, definitions: Vec::new(),
                status: WordStatus::from_str(&row.get::<_, String>(4)?),
                created_at: row.get(5)?, updated_at: row.get(6)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        drop(stmt);
        drop(conn);
        let mut result = Vec::new();
        for mut word in words {
            if let Ok(defs) = self.get_definitions(&word.id) {
                word.definitions = defs;
            }
            result.push(word);
        }
        Ok(result)
    }

    pub fn get_setting(&self, key: &str) -> SqliteResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else { Ok(None) }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)", params![key, value])?;
        Ok(())
    }
}