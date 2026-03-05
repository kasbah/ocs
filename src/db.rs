use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub directory: String,
    pub time_created: i64, // epoch milliseconds
    pub last_input: String,
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

/// Query top-level sessions with the last user text input for each.
pub fn query_sessions() -> Result<Vec<Session>, String> {
    let path = db_path()?;
    let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Failed to open database: {e}"))?;

    // For each session, find the last user message and get its first text part.
    let query = "
        SELECT
            s.id,
            s.title,
            s.directory,
            s.time_created,
            COALESCE(json_extract(p.data, '$.text'), '') AS last_input
        FROM session s
        LEFT JOIN (
            SELECT m.session_id, m.id AS msg_id
            FROM message m
            WHERE json_extract(m.data, '$.role') = 'user'
              AND m.time_created = (
                SELECT MAX(m2.time_created)
                FROM message m2
                WHERE m2.session_id = m.session_id
                  AND json_extract(m2.data, '$.role') = 'user'
              )
        ) last_msg ON last_msg.session_id = s.id
        LEFT JOIN (
            SELECT p.message_id, p.data,
                   ROW_NUMBER() OVER (PARTITION BY p.message_id ORDER BY p.time_created ASC) AS rn
            FROM part p
            WHERE json_extract(p.data, '$.type') = 'text'
        ) p ON p.message_id = last_msg.msg_id AND p.rn = 1
        WHERE s.parent_id IS NULL
        ORDER BY s.time_created DESC
    ";

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("Query error: {e}"))?;

    let sessions = stmt
        .query_map([], |row| {
            let last_input: String = row.get(4)?;
            // Take only the first line, truncated to a reasonable length
            let first_line = last_input.lines().next().unwrap_or("").to_string();
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                directory: row.get(2)?,
                time_created: row.get(3)?,
                last_input: first_line,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {e}"))?;

    Ok(sessions)
}
