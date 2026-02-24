use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a task in the application.
pub struct Task {
    /// The unique identifier for the task.
    pub id: Uuid,
    /// The title of the task.
    pub title: String,
    /// Notes about the task.
    pub notes: String,
    /// Whether the task is marked as a favorite.
    pub favorite: bool,
    /// Whether the task is marked for today.
    pub today: bool,
    /// Whether the task is expanded in the UI.
    pub expanded: bool,
    /// The status of the task (e.g., not started, completed).
    pub status: Status,
    /// The priority level of the task (e.g., low, normal, high).
    pub priority: Priority,
    /// The recurrence pattern for the task (e.g., which days of the week it recurs on).
    pub recurrence: Recurrence,
    /// The tags associated with the task.
    pub tags: Vec<String>,
    /// The parent task ID (None if this is a top-level task).
    pub parent_id: Option<Uuid>,
    /// The IDs of direct child tasks.
    pub sub_task_ids: Vec<Uuid>,
    /// The date and time when the task was completed, if applicable.
    pub completion_date: Option<DateTime<Utc>>,
    /// The date and time when the task is due, if applicable.
    pub due_date: Option<DateTime<Utc>>,
    /// The date and time when the task is scheduled, if applicable.
    pub reminder_date: Option<DateTime<Utc>>,
    /// The date and time when the task was created.
    pub creation_date: DateTime<Utc>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            notes: String::new(),
            favorite: false,
            today: false,
            expanded: false,
            status: Status::NotStarted,
            priority: Priority::Normal,
            recurrence: Recurrence::default(),
            tags: Vec::new(),
            parent_id: None,
            sub_task_ids: Vec::new(),
            completion_date: None,
            due_date: None,
            reminder_date: None,
            creation_date: Utc::now(),
        }
    }
}

impl Task {
    /// Creates a new task with the given title and default values for other fields.
    pub fn new(title: impl ToString) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            notes: String::new(),
            favorite: false,
            today: false,
            expanded: false,
            status: Status::NotStarted,
            priority: Priority::Normal,
            recurrence: Recurrence::default(),
            tags: Vec::new(),
            parent_id: None,
            sub_task_ids: Vec::new(),
            completion_date: None,
            due_date: None,
            reminder_date: None,
            creation_date: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    #[default]
    /// The task has not been started yet.
    NotStarted,
    /// The task is completed.
    Completed,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Priority {
    #[default]
    /// The task has low priority.
    Low,
    /// The task has normal priority.
    Normal,
    /// The task has high priority.
    High,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Recurrence {
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
}
