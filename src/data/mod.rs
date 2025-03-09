use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Activity {
    pub id: Option<i64>,
    pub name: String,
    pub category: String,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub duration: Option<Duration>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub is_productive: bool,
}

impl Activity {
    pub fn new(name: String, category: String, tags: Vec<String>, is_productive: bool) -> Self {
        Self {
            id: None,
            name,
            category,
            start_time: Local::now(),
            end_time: None,
            duration: None,
            tags,
            notes: None,
            is_productive,
        }
    }
    
    pub fn stop(&mut self) {
        let now = Local::now();
        self.end_time = Some(now);
        self.duration = Some(now.signed_duration_since(self.start_time).to_std().unwrap_or_default());
    }
}

