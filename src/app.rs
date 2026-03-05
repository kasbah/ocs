use crate::db::Session;
use crate::fuzzy::{filter_sessions, ScoredSession};

/// The result of running the app — either the user selected a session or quit.
pub enum AppResult {
    Selected(Session),
    Quit,
}

pub struct App {
    pub sessions: Vec<Session>,
    pub filtered: Vec<ScoredSession>,
    pub query: String,
    pub selected: usize,
    pub result: Option<AppResult>,
}

impl App {
    pub fn new(sessions: Vec<Session>) -> Self {
        let filtered = filter_sessions(&sessions, "");
        App {
            sessions,
            filtered,
            query: String::new(),
            selected: 0,
            result: None,
        }
    }

    /// Re-filter sessions based on current query.
    fn update_filter(&mut self) {
        self.filtered = filter_sessions(&self.sessions, &self.query);
        // Clamp selection
        if self.filtered.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len() - 1;
        }
    }

    /// Append a character to the query.
    pub fn type_char(&mut self, c: char) {
        self.query.push(c);
        self.update_filter();
    }

    /// Delete the last character from the query.
    pub fn backspace(&mut self) {
        self.query.pop();
        self.update_filter();
    }

    /// Move selection up.
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down.
    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected < self.filtered.len() - 1 {
            self.selected += 1;
        }
    }

    /// Confirm the current selection.
    pub fn confirm(&mut self) {
        if let Some(scored) = self.filtered.get(self.selected) {
            self.result = Some(AppResult::Selected(scored.session.clone()));
        }
    }

    /// Quit without selecting.
    pub fn quit(&mut self) {
        self.result = Some(AppResult::Quit);
    }

    /// Whether the app should exit.
    pub fn should_exit(&self) -> bool {
        self.result.is_some()
    }
}
