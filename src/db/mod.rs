use rusqlite::{Connection, Result};
use std::path::PathBuf;
use dirs::home_dir;
use crate::data::Activity;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }
    
    fn get_db_path() -> PathBuf {
        let mut path = home_dir().unwrap_or_default();
        path.push(".productivity_tracker");
        std::fs::create_dir_all(&path).unwrap_or_default();
        path.push("activities.db");
        path
    }
    
    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS activities (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                duration INTEGER,
                tags TEXT NOT NULL,
                notes TEXT,
                is_productive INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }
    
    pub fn save_activity(&self, activity: &Activity) -> Result<i64> {
        let tags_json = serde_json::to_string(&activity.tags).unwrap_or_default();
        let id = self.conn.execute(
            "INSERT INTO activities (name, category, start_time, end_time, duration, tags, notes, is_productive)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &activity.name,
                &activity.category,
                &activity.start_time.to_rfc3339(),
                &activity.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
                &activity.duration.map(|d| d.as_secs() as i64).unwrap_or_default().to_string(),
                &tags_json,
                &activity.notes.clone().unwrap_or_default(),
                &(if activity.is_productive { 1 } else { 0 }).to_string(),
            ],
        )?;
        Ok(id as i64)
    }
    
    // Additional methods for querying and updating activities will be added here
}

