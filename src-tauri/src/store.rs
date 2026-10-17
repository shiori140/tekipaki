use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Now,
    Today,
    Later,
}

impl Priority {
    /// 連打回数から優先度を決める。4回目以降は「今すぐ」に戻る。
    pub fn from_count(count: u32) -> Self {
        match (count.max(1) - 1) % 3 {
            0 => Priority::Now,
            1 => Priority::Today,
            _ => Priority::Later,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    pub memo: String,
    pub priority: Priority,
    pub status: Status,
    pub source_app: Option<String>,
    pub created_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
}

impl Task {
    /// 取得した文の1行目をタイトル、2行目以降をメモにする。
    pub fn from_text(text: &str, priority: Priority, source_app: Option<String>) -> Option<Self> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let (title, memo) = match text.split_once('\n') {
            Some((first, rest)) => (first.trim(), rest.trim()),
            None => (text, ""),
        };
        Some(Task {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            memo: memo.to_string(),
            priority,
            status: Status::Open,
            source_app,
            created_at: Local::now(),
            completed_at: None,
        })
    }
}

pub struct Store {
    path: PathBuf,
    tasks: Mutex<Vec<Task>>,
}

impl Store {
    pub fn open(dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&dir)?;
        let path = dir.join("tasks.json");
        let tasks = match fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
                // 壊れたファイルを上書きして消さないよう、退避してから空で始める。
                eprintln!("tasks.json の読み込みに失敗しました: {e}");
                let _ = fs::rename(&path, dir.join("tasks.broken.json"));
                Vec::new()
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };
        Ok(Store {
            path,
            tasks: Mutex::new(tasks),
        })
    }

    pub fn add(&self, task: Task) -> std::io::Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
        self.write(&tasks)
    }

    pub fn list(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().clone()
    }

    fn write(&self, tasks: &[Task]) -> std::io::Result<()> {
        // 書き込み途中で落ちてもデータが消えないよう、一時ファイルに書いてから置き換える。
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(tasks)?)?;
        fs::rename(&tmp, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_cycles_by_press_count() {
        assert_eq!(Priority::from_count(1), Priority::Now);
        assert_eq!(Priority::from_count(2), Priority::Today);
        assert_eq!(Priority::from_count(3), Priority::Later);
        assert_eq!(Priority::from_count(4), Priority::Now);
    }

    #[test]
    fn splits_title_and_memo() {
        let t = Task::from_text("  見積もり返信\n金額は来週\n確定  ", Priority::Now, None).unwrap();
        assert_eq!(t.title, "見積もり返信");
        assert_eq!(t.memo, "金額は来週\n確定");
        assert!(Task::from_text("  \n ", Priority::Now, None).is_none());
    }

    #[test]
    fn persists_and_reloads() {
        let dir = std::env::temp_dir().join(format!("tekipaki-test-{}", uuid::Uuid::new_v4()));
        let store = Store::open(dir.clone()).unwrap();
        store
            .add(Task::from_text("a", Priority::Later, Some("Slack".into())).unwrap())
            .unwrap();
        let reloaded = Store::open(dir.clone()).unwrap().list();
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0].priority, Priority::Later);
        assert_eq!(reloaded[0].source_app.as_deref(), Some("Slack"));
        fs::remove_dir_all(dir).unwrap();
    }
}
