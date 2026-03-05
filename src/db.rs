use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub directory: String,
    pub time_created: i64, // epoch milliseconds
}

/// Locate the opencode SQLite database.
pub fn db_path() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir().ok_or("Could not determine XDG data directory")?;
    let path = data_dir.join("opencode").join("opencode.db");
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Database not found at {}", path.display()))
    }
}

/// Query top-level sessions (no subagent children) ordered by date descending.
pub fn query_sessions() -> Result<Vec<Session>, String> {
    let path = db_path()?;
    let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Failed to open database: {e}"))?;

    let query = "SELECT id, title, directory, time_created \
                 FROM session \
                 WHERE parent_id IS NULL \
                 ORDER BY time_created DESC";

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("Query error: {e}"))?;

    let sessions = stmt
        .query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                directory: row.get(2)?,
                time_created: row.get(3)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))?;

    Ok(sessions)
}
