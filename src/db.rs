use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        // Get the user's data directory
        let mut data_dir = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.push("rusty_pomodoro");
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&data_dir)?;
        
        // Create database file path
        let mut db_path = data_dir;
        db_path.push("sessions.db");
        
        // Connect to database
        let conn = Connection::open(db_path)?;
        
        Ok(Database { conn })
    }
    
    pub fn initialize(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                pomodoro_count INTEGER NOT NULL,
                completed BOOLEAN NOT NULL
            )",
            [],
        )?;
        
        Ok(())
    }
    
    pub fn save_session(
        &self, 
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
        pomodoro_count: u64,
        completed: bool,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (start_time, end_time, pomodoro_count, completed) 
             VALUES (?, ?, ?, ?)",
            params![
                start_time.to_rfc3339(),
                end_time.to_rfc3339(),
                pomodoro_count as i64,
                completed,
            ],
        )?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    
    #[test]
    fn test_db_create_and_query() -> Result<()> {
        // Use in-memory database for testing
        let conn = Connection::open(":memory:")?;
        let db = Database { conn };
        
        // Initialize schema
        db.initialize()?;
        
        // Add a test session
        let now = Local::now();
        let end = now + Duration::minutes(25);
        
        db.save_session(now, end, 1, true)?;
        
        // Query for saved session
        let mut stmt = db.conn.prepare("SELECT * FROM sessions")?;
        let mut rows = stmt.query([])?;
        
        let row = rows.next()?.unwrap();
        let completed: bool = row.get(4)?;
        
        assert!(completed);
        
        Ok(())
    }
}