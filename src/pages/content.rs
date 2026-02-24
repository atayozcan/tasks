use std::collections::HashMap;

use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Alignment, Length, Subscription,
    },
    iced_widget::row,
    theme,
    widget::{self, menu::Action as MenuAction},
    Apply, Element,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use crate::{
    core::config,
    fl,
    model::{self, List, Status},
    services::store::Store,
};

/// Represents the edit state of an input field to prevent feedback loops
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
enum EditState {
    /// Input is not in edit mode
    #[default]
    Idle,
    /// Edit mode requested, waiting to focus
    Entering,
    /// Input is actively being edited
    Editing,
    /// Edit mode exit requested, waiting to blur
    Exiting,
}

pub struct Content {
    selected_list: Option<List>,
    tasks: SlotMap<DefaultKey, model::Task>,
    editing: SecondaryMap<DefaultKey, EditState>,
    inputs: SecondaryMap<DefaultKey, widget::Id>,
    config: config::AppConfig,
    store: Store,

    context_menu_open: bool,
    search_bar_visible: bool,
    add_task_input: String,
    search_query: String,
    sort_type: SortType,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SortType {
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
}

#[derive(Debug, Clone)]
pub enum Message {
    TaskAdd,

    TaskExpand(DefaultKey),
    TaskAddSubTask(DefaultKey),
    TaskComplete(DefaultKey, bool),
    TaskDelete(DefaultKey),
    TaskToggleTitleEditMode(DefaultKey, bool),
    TaskTitleInput(String),
    TaskOpenDetails(DefaultKey),
    TaskTitleSubmit(DefaultKey),
    TaskTitleUpdate(DefaultKey, String),

    ToggleHideCompleted,

    SetList(Option<List>),
    SetTasks(Vec<model::Task>),
    SetConfig(config::AppConfig),
    RefreshTask(model::Task),
    Empty,
    ContextMenuOpen(bool),

    ToggleSearchBar,
    SearchQueryChanged(String),
    SetSort(SortType),
}

pub enum Output {
    ToggleHideCompleted(model::List),
    Focus(widget::Id),
    OpenTaskDetails(model::Task),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskAction {
    AddSubTask(DefaultKey),
    Edit(DefaultKey),
    Delete(DefaultKey),
}

impl MenuAction for TaskAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            TaskAction::Edit(id) => Message::TaskOpenDetails(*id),
            TaskAction::AddSubTask(id) => Message::TaskAddSubTask(*id),
            TaskAction::Delete(id) => Message::TaskDelete(*id),
        }
    }
}

impl Content {
    pub fn new(storage: Store, config: config::AppConfig) -> Self {
        Self {
            selected_list: None,
            tasks: SlotMap::new(),
            editing: SecondaryMap::new(),
            inputs: SecondaryMap::new(),
            add_task_input: String::new(),
            config: config,
            store: storage,
            context_menu_open: false,
            search_bar_visible: false,
            search_query: String::new(),
            sort_type: SortType::DateAsc,
        }
    }

    fn list_header<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let hide_completed_active = list.hide_completed || self.config.hide_completed;
        let mut hide_completed_button =
            widget::button::icon(widget::icon::from_name("check-round-outline-symbolic").size(18))
                .selected(hide_completed_active)
                .padding(spacing.space_xxs);

        if hide_completed_active {
            hide_completed_button = hide_completed_button.class(cosmic::style::Button::Suggested);
        }

        hide_completed_button = hide_completed_button.on_press(Message::ToggleHideCompleted);

        let search_button =
            widget::button::icon(widget::icon::from_name("edit-find-symbolic").size(18))
                .selected(self.search_bar_visible)
                .padding(spacing.space_xxs)
                .on_press(Message::ToggleSearchBar);

        let icon = widget::icon::from_name(list.icon.as_deref().unwrap_or("view-list-symbolic"))
            .size(spacing.space_m);
        widget::row::with_capacity(4)
            .align_y(Alignment::Center)
            .spacing(spacing.space_s)
            .padding([spacing.space_none, spacing.space_xxs])
            .push(icon)
            .push(widget::text::body(&list.name).size(24).width(Length::Fill))
            .push(hide_completed_button)
            .push(search_button)
            .into()
    }

    pub fn list_view<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut column = widget::column::with_capacity(3);
        column = column.push(self.list_header(list));

        if self.search_bar_visible {
            column = column.push(
                widget::text_input(fl!("search-tasks"), &self.search_query)
                    .on_input(Message::SearchQueryChanged)
                    .width(Length::Fill)
                    .padding([spacing.space_xxs, spacing.space_xxs]),
            );
        }

        let mut tasks_vec: Vec<_> = self.tasks.iter().collect();
        match self.sort_type {
            SortType::NameAsc => {
                tasks_vec.sort_by(|a, b| a.1.title.to_lowercase().cmp(&b.1.title.to_lowercase()))
            }
            SortType::NameDesc => {
                tasks_vec.sort_by(|a, b| b.1.title.to_lowercase().cmp(&a.1.title.to_lowercase()))
            }
            SortType::DateAsc => {
                tasks_vec.sort_by(|a, b| a.1.creation_date.cmp(&b.1.creation_date))
            }
            SortType::DateDesc => {
                tasks_vec.sort_by(|a, b| b.1.creation_date.cmp(&a.1.creation_date))
            }
        }

        let filtered_tasks: Vec<_> = tasks_vec
            .into_iter()
            .filter(|(_, task)| {
                // Only show top-level tasks (no parent)
                task.parent_id.is_none()
                // Search filter
                && (!self.search_bar_visible || self.search_query.is_empty() || task.title.to_lowercase().contains(&self.search_query.to_lowercase()))
                // Hide completed filter
                && (!(list.hide_completed || self.config.hide_completed) || task.status != Status::Completed)
            })
            .map(|(id, task)| self.task_view(id, task))
            .collect();

        if filtered_tasks.is_empty() && self.search_query.is_empty() {
            return self.empty(list);
        }

        let items = widget::column::with_children(filtered_tasks).spacing(spacing.space_s);

        column
            .push(items)
            .padding([spacing.space_none, spacing.space_l])
            .spacing(spacing.space_s)
            .apply(widget::container)
            .height(Length::Shrink)
            .apply(widget::scrollable)
            .height(Length::Fill)
            .into()
    }

    pub fn task_view<'a>(&'a self, id: DefaultKey, task: &'a model::Task) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        // Get direct children of this task
        let sub_tasks: Vec<_> = self
            .tasks
            .iter()
            .filter(|(_, sub_task)| sub_task.parent_id == Some(task.id))
            .collect();

        let item_checkbox = widget::checkbox("", task.status == Status::Completed)
            .on_toggle(move |value| Message::TaskComplete(id, value));

        let not_empty = !sub_tasks.is_empty();
        let icon = if task.expanded {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        };
        let expand_button = not_empty.then(|| {
            widget::button::icon(widget::icon::from_name(icon).size(18))
                .padding(spacing.space_xxs)
                .on_press(Message::TaskExpand(id))
        });

        let more_button = widget::menu::MenuBar::new(vec![widget::menu::Tree::with_children(
            Element::from(
                cosmic::widget::button::icon(
                    widget::icon::from_name("view-more-symbolic").size(18),
                )
                .on_press(Message::Empty),
            ),
            widget::menu::items(
                &HashMap::new(),
                vec![
                    widget::menu::Item::Button(fl!("edit"), None, TaskAction::Edit(id)),
                    widget::menu::Item::Button(
                        fl!("add-sub-task"),
                        None,
                        TaskAction::AddSubTask(id),
                    ),
                    widget::menu::Item::Button(fl!("delete"), None, TaskAction::Delete(id)),
                ],
            ),
        )])
        .item_height(widget::menu::ItemHeight::Dynamic(40))
        .item_width(widget::menu::ItemWidth::Uniform(260))
        .spacing(4.0);

        let (completed, total) = sub_tasks.iter().fold((0, 0), |acc, (_, subtask)| {
            if subtask.status == Status::Completed {
                (acc.0 + 1, acc.1 + 1)
            } else {
                (acc.0, acc.1 + 1)
            }
        });

        let subtask_count = if total > 0 {
            Some(widget::text(format!("{}/{}", completed, total)))
        } else {
            None
        };

        let task_item_text = widget::editable_input(
            "",
            &task.title,
            matches!(
                self.editing.get(id),
                Some(EditState::Entering) | Some(EditState::Editing)
            ),
            move |editing| Message::TaskToggleTitleEditMode(id, editing),
        )
        .size(13)
        .trailing_icon(widget::column().into())
        .id(self.inputs[id].clone())
        .on_submit(move |_| Message::TaskTitleSubmit(id))
        .on_input(move |text| Message::TaskTitleUpdate(id, text));

        let row = widget::row::with_capacity(5)
            .align_y(Alignment::Center)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_xxs, spacing.space_s])
            .push(item_checkbox)
            .push(task_item_text)
            .push_maybe(expand_button)
            .push_maybe(subtask_count)
            .push(more_button);

        let mut column = widget::column::with_capacity(2).push(row);

        if task.expanded && !sub_tasks.is_empty() {
            let subtask_elements = sub_tasks
                .iter()
                .map(|(sub_id, sub_task)| {
                    widget::container(self.task_view(*sub_id, sub_task))
                        .padding([0, 0, 0, spacing.space_xs])
                        .into()
                })
                .collect::<Vec<_>>();
            column = column.push(widget::column::with_children(subtask_elements));
        }

        widget::container(column)
            .class(cosmic::style::Container::ContextDrawer)
            .into()
    }

    pub fn empty<'a>(&'a self, list: &'a List) -> Element<'a, Message> {
        let spacing = theme::active().cosmic().spacing;

        let container = widget::container(
            widget::column::with_children(vec![
                widget::icon::from_name("task-past-due-symbolic")
                    .size(56)
                    .into(),
                widget::text::title1(fl!("no-tasks")).into(),
                widget::text(fl!("no-tasks-suggestion")).into(),
            ])
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .align_y(Vertical::Center)
        .align_x(Horizontal::Center)
        .height(Length::Fill)
        .width(Length::Fill);

        widget::column::with_capacity(2)
            .push(self.list_header(list))
            .push(container)
            .padding([spacing.space_none, spacing.space_l])
            .spacing(spacing.space_s)
            .into()
    }

    pub fn new_task_view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;
        row(vec![
            widget::text_input(fl!("add-new-task"), &self.add_task_input)
                .id(widget::Id::new("new-task-input"))
                .on_input(Message::TaskTitleInput)
                .on_submit(|_| Message::TaskAdd)
                .width(Length::Fill)
                .into(),
            widget::button::icon(widget::icon::from_name("mail-send-symbolic").size(18))
                .padding(spacing.space_xxs)
                .class(cosmic::style::Button::Suggested)
                .on_press(Message::TaskAdd)
                .into(),
        ])
        .padding(spacing.space_xxs)
        .spacing(spacing.space_xxs)
        .align_y(Alignment::Center)
        .into()
    }

    fn populate_task_slotmap(&mut self, tasks: Vec<model::Task>) {
        for task in tasks {
            let task_id = self.tasks.insert(task);
            self.inputs.insert(task_id, widget::Id::unique());
            self.editing.insert(task_id, EditState::Idle);
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Output> {
        let mut output = None;
        match message {
            Message::ToggleSearchBar => {
                self.search_bar_visible = !self.search_bar_visible;
                if !self.search_bar_visible {
                    self.search_query.clear();
                }
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
            }
            Message::Empty => (),
            Message::ContextMenuOpen(open) => {
                self.context_menu_open = open;
            }
            Message::SetTasks(tasks) => {
                self.tasks.clear();
                self.inputs.clear();
                self.editing.clear();
                self.add_task_input.clear();
                self.populate_task_slotmap(tasks);
            }
            Message::SetList(list) => {
                match (&self.selected_list, &list) {
                    (Some(current), Some(list)) => {
                        if current.id != list.id {
                            match self.store.tasks(list.id).load_all() {
                                Ok(tasks) => {
                                    self.update(Message::SetTasks(tasks));
                                }
                                Err(error) => {
                                    tracing::error!("Failed to fetch tasks for list: {:?}", error)
                                }
                            }
                        }
                    }
                    (None, Some(list)) => match self.store.tasks(list.id).load_all() {
                        Ok(tasks) => {
                            self.update(Message::SetTasks(tasks));
                        }
                        Err(error) => {
                            tracing::error!("Failed to fetch tasks for list: {:?}", error)
                        }
                    },
                    _ => {}
                }
                self.selected_list.clone_from(&list);
            }
            Message::SetConfig(config) => {
                self.config = config;
            }
            Message::RefreshTask(refreshed_task) => {
                if let Some((id, _)) = self.tasks.iter().find(|(_, t)| t.id == refreshed_task.id) {
                    if let Some(task) = self.tasks.get_mut(id) {
                        *task = refreshed_task.clone();
                    }
                } else {
                    tracing::warn!("Task with ID {:?} not found", refreshed_task.id);
                }
            }
            Message::TaskOpenDetails(id) => match self.tasks.get(id) {
                Some(task) => output = Some(Output::OpenTaskDetails(task.clone())),
                None => tracing::warn!("Task with ID {:?} not found", id),
            },
            Message::TaskExpand(default_key) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };
                if let Some(task) = self.tasks.get_mut(default_key) {
                    task.expanded = !task.expanded;
                    if let Err(error) = self
                        .store
                        .tasks(list.id)
                        .update(task.id, |task| task.expanded = !task.expanded)
                    {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::TaskAdd => {
                if let Some(list) = &self.selected_list {
                    if !self.add_task_input.is_empty() {
                        let task = model::Task::new(self.add_task_input.clone());
                        match self.store.tasks(list.id).save(&task) {
                            Ok(_) => {
                                let id = self.tasks.insert(task);
                                self.inputs.insert(id, widget::Id::unique());
                                self.add_task_input.clear();
                            }
                            Err(error) => {
                                tracing::error!("Failed to create task: {:?}", error);
                            }
                        }
                    }
                }
            }
            Message::TaskToggleTitleEditMode(id, editing) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                let current_state = self.editing.get(id).copied().unwrap_or_default();

                // State machine to prevent feedback loops
                let new_state = match (current_state, editing) {
                    // Request to enter edit mode from idle state
                    (EditState::Idle, true) => {
                        output = Some(Output::Focus(self.inputs[id].clone()));
                        Some(EditState::Entering)
                    }
                    // Confirmation that edit mode was entered (from widget after focus)
                    (EditState::Entering, true) => Some(EditState::Editing),
                    // Request to exit edit mode
                    (EditState::Editing, false) => {
                        if let Some(task) = self.tasks.get(id) {
                            if let Err(error) = self
                                .store
                                .tasks(list.id)
                                .update(task.id, |t| *t = task.clone())
                            {
                                tracing::error!("Failed to update task: {:?}", error);
                            }
                        }
                        Some(EditState::Exiting)
                    }
                    // Confirmation that edit mode was exited (from widget after blur)
                    (EditState::Exiting, false) => Some(EditState::Idle),
                    // Ignore redundant state changes that would cause loops
                    (EditState::Entering, false) | (EditState::Exiting, true) => {
                        tracing::debug!("Ignoring redundant edit state change for task {:?}", id);
                        None
                    }
                    // Already in requested state, ignore
                    (EditState::Idle, false) | (EditState::Editing, true) => None,
                };

                if let Some(state) = new_state {
                    self.editing.insert(id, state);
                }
            }
            Message::TaskTitleInput(input) => self.add_task_input = input,
            Message::TaskTitleSubmit(id) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                if let Some(task) = self.tasks.get(id) {
                    match self
                        .store
                        .tasks(list.id)
                        .update(task.id, |t| *t = task.clone())
                    {
                        Ok(_) => {
                            self.editing.insert(id, EditState::Idle);
                            output = Some(Output::Focus(widget::Id::new("new-task-input")));
                        }
                        Err(error) => tracing::error!("Failed to update task: {:?}", error),
                    }
                }
            }
            Message::TaskTitleUpdate(id, title) => {
                if let Some(task) = self.tasks.get_mut(id) {
                    task.title = title;
                }
            }
            Message::TaskDelete(id) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                if let Some(task) = self.tasks.remove(id) {
                    if let Err(error) = self.store.tasks(list.id).delete(task.id) {
                        tracing::error!("Failed to delete task: {:?}", error);
                    }
                }
            }
            Message::TaskComplete(id, complete) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                let task = self.tasks.get_mut(id);
                if let Some(task) = task {
                    task.status = if complete {
                        Status::Completed
                    } else {
                        Status::NotStarted
                    };
                    if let Err(error) = self
                        .store
                        .tasks(list.id)
                        .update(task.id, |t| *t = task.clone())
                    {
                        tracing::error!("Failed to update task: {:?}", error);
                    }
                }
            }
            Message::TaskAddSubTask(id) => {
                let Some(list) = &self.selected_list else {
                    tracing::warn!("No list selected");
                    return None;
                };

                if let Some(task) = self.tasks.get_mut(id) {
                    task.expanded = true;
                    let mut sub_task = model::Task::new("".to_string());
                    sub_task.parent_id = Some(task.id);

                    match self.store.tasks(list.id).save(&sub_task) {
                        Ok(_) => {
                            // Add sub_task ID to parent's sub_task_ids
                            task.sub_task_ids.push(sub_task.id);
                            if let Err(error) = self
                                .store
                                .tasks(list.id)
                                .update(task.id, |t| *t = task.clone())
                            {
                                tracing::error!("Failed to update task with sub-task: {:?}", error);
                            }

                            // Insert subtask into the same tasks slotmap
                            let sub_task_id = self.tasks.insert(sub_task);
                            self.inputs.insert(sub_task_id, widget::Id::unique());
                            self.editing.insert(sub_task_id, EditState::Entering);
                            output = Some(Output::Focus(self.inputs[sub_task_id].clone()));
                        }
                        Err(error) => {
                            tracing::error!("Failed to add sub-task: {:?}", error);
                        }
                    }
                }
            }
            Message::ToggleHideCompleted => {
                if let Some(ref mut list) = self.selected_list {
                    match self.store.lists().update(list.id, |list| {
                        list.hide_completed = !list.hide_completed;
                    }) {
                        Ok(updated) => {
                            list.hide_completed = !updated.hide_completed;
                            output = Some(Output::ToggleHideCompleted(updated.clone()));
                        }
                        Err(err) => {
                            tracing::error!("Error updating list: {err}");
                        }
                    }
                }
            }
            Message::SetSort(sort_type) => {
                self.sort_type = sort_type;
            }
        }
        output
    }

    pub fn view(&self) -> Element<'_, Message> {
        let spacing = theme::active().cosmic().spacing;

        let Some(ref list) = self.selected_list else {
            return widget::container(
                widget::column::with_children(vec![
                    widget::icon::from_name("applications-office-symbolic")
                        .size(56)
                        .into(),
                    widget::text::title1(fl!("no-list-selected")).into(),
                    widget::text(fl!("no-list-suggestion")).into(),
                ])
                .spacing(10)
                .align_x(Alignment::Center),
            )
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();
        };

        widget::column::with_capacity(2)
            .push(self.list_view(list))
            .push(self.new_task_view())
            .spacing(spacing.space_xxs)
            .max_width(800.)
            .apply(widget::container)
            .height(Length::Fill)
            .width(Length::Fill)
            .center(if self.context_menu_open {
                Length::Shrink
            } else {
                Length::Fill
            })
            .padding([spacing.space_xxs, spacing.space_none])
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
