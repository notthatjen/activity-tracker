use crate::data::Activity;
use crate::db::Database;
use rusqlite::Result;

pub struct Tracker {
    db: Database,
    current_activity: Option<Activity>,
}

impl Tracker {
    pub fn new() -> Result<Self> {
        let db = Database::new()?;
        Ok(Self {
            db,
            current_activity: None,
        })
    }
    
    pub fn start_activity(&mut self, name: String, category: String, tags: Vec<String>, is_productive: bool) -> Result<()> {
        if self.current_activity.is_some() {
            self.stop_activity()?;
        }
        
        let activity = Activity::new(name, category, tags, is_productive);
        self.current_activity = Some(activity);
        Ok(())
    }
    
    pub fn stop_activity(&mut self) -> Result<()> {
        if let Some(mut activity) = self.current_activity.take() {
            activity.stop();
            self.db.save_activity(&activity)?;
        }
        Ok(())
    }
    
    pub fn get_current_activity(&self) -> Option<&Activity> {
        self.current_activity.as_ref()
    }
    
    // Additional tracker methods will be added here
}

