use crate::{
    app::{context::ContextPage, Message},
    model::List,
};
use cosmic::{
    iced::keyboard::{Key, Modifiers},
    widget::{self, menu::Action, segmented_button},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
    WindowClose,
    WindowNew,
    NewList,
    DeleteList,
    RenameList,
    Icon,
    ToggleHideCompleted(bool),
    SortByNameAsc,
    SortByNameDesc,
    SortByDateAsc,
    SortByDateDesc,
}

#[derive(Debug, Clone)]
pub enum ApplicationAction {
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    AppTheme(usize),
    Focus(widget::Id),
}

#[derive(Debug, Clone)]
pub enum TasksAction {
    PopulateLists(Vec<List>),
    AddList(List),
    DeleteList(Option<segmented_button::Entity>),
    FetchLists,
}

impl Action for MenuAction {
    type Message = Message;
    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            action => Message::Menu(*action),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NavMenuAction {
    Rename(segmented_button::Entity),
    SetIcon(segmented_button::Entity),
    Export(segmented_button::Entity),
    Delete(segmented_button::Entity),
}

impl Action for NavMenuAction {
    type Message = cosmic::Action<Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(Message::NavMenu(*self))
    }
}
