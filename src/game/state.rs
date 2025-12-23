use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Waiting,
    Night,
    Day,
    Voting,
    Ended,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub night_count: i32,
    pub phase: Phase,
    pub log: Vec<String>,
    max_log_entries: usize,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            night_count: 0,
            phase: Phase::Waiting,
            log: Vec::new(),
            max_log_entries: 100,
        }
    }

    pub fn add_log(&mut self, entry: String) {
        self.log.push(entry);
        if self.log.len() > self.max_log_entries {
            self.log = self.log[self.log.len() - self.max_log_entries..].to_vec();
        }
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    pub fn reset(&mut self) {
        self.night_count = 0;
        self.phase = Phase::Waiting;
        self.clear_log();
    }

    pub fn reset_to_night(&mut self) {
        self.phase = Phase::Night;
        self.night_count += 1;
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
