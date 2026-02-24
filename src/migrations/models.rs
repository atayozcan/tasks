use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::model::{Priority, Recurrence, Status};

/// Old Task structure with nested sub-tasks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    #[serde(default)]
    pub path: PathBuf,
    pub title: String,
    pub favorite: bool,
    pub today: bool,
    pub status: Status,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub notes: String,
    pub completion_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub reminder_date: Option<DateTime<Utc>>,
    pub recurrence: Recurrence,
    #[serde(default)]
    pub expanded: bool,
    pub sub_tasks: Vec<Task>,
    pub deletion_date: Option<DateTime<Utc>>,
    pub created_date_time: DateTime<Utc>,
    pub last_modified_date_time: DateTime<Utc>,
}

/// Old List structure with file_path
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List {
    pub id: String,
    #[serde(default)]
    pub file_path: PathBuf,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub hide_completed: bool,
}
