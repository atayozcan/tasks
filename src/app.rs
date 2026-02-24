pub mod actions;
pub mod context;
pub mod dialog;
pub mod flags;
pub mod markdown;
pub mod menu;

pub use flags::*;
use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
    env, process,
};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use cosmic::{
    app::{self, Core},
    cosmic_config::{self, Update},
    cosmic_theme::{self, ThemeMode},
    iced::{
        keyboard::{Event as KeyEvent, Modifiers},
        Event, Subscription,
    },
    widget::{
        self,
        about::About,
        calendar::CalendarModel,
        menu::{key_bind::KeyBind, Action as _},
        nav_bar,
        segmented_button::{Entity, EntityMut, SingleSelect},
    },
    Application, ApplicationExt, Element,
};

use crate::{
    app::{
        actions::{ApplicationAction, MenuAction, NavMenuAction, TasksAction},
        context::ContextPage,
        dialog::{DialogAction, DialogPage},
        markdown::Markdown,
    },
    core::{
        config::{self, CONFIG_VERSION},
        key_bind::key_binds,
    },
    fl,
    model::List,
    pages::{
        content::{self, Content},
        details::{self, Details},
    },
    services::store::Store,
};

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<KeyBind, MenuAction>,
    /// Configuration handler for managing app settings.
    handler: cosmic_config::Config,
    /// Application-specific configuration.
    config: config::AppConfig,

    /// Current keyboard modifiers.
    modifiers: Modifiers,
    /// Queue of dialog pages.
    dialog_pages: VecDeque<DialogPage>,
    /// Identifier for the dialog text input widget.
    dialog_text_input: widget::Id,

    /// Persistent storage for lists and tasks.
    store: Store,
    /// The main content area of the application.
    content: Content,
    /// The details view for tasks.
    details: Details,
}

#[derive(Debug, Clone)]
pub enum Message {
    Content(content::Message),
    Details(details::Message),
    Tasks(TasksAction),
    Menu(MenuAction),
    Dialog(DialogAction),
    NavMenu(NavMenuAction),
    Application(ApplicationAction),
    ToggleContextDrawer,
    ToggleContextPage(ContextPage),
    Open(String),
}

impl Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = crate::app::Flags;
    type Message = Message;
    const APP_ID: &'static str = "dev.edfloreshz.Tasks";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, app::Task<Self::Message>) {
        let nav_model = widget::segmented_button::ModelBuilder::default().build();

        let about = widget::about::About::default()
            .name(fl!("tasks"))
            .icon(widget::icon::from_name(Self::APP_ID))
            .version("0.2.0")
            .author("Eduardo Flores")
            .license("GPL-3.0-only")
            .links([
                (fl!("repository"), "https://github.com/cosmic-utils/tasks"),
                (
                    fl!("support"),
                    "https://github.com/cosmic-utils/tasks/issues",
                ),
                (fl!("website"), "https://tasks.edfloreshz.dev"),
            ])
            .developers([("Eduardo Flores", "edfloreshz@proton.me")]);

        let mut app = AppModel {
            core,
            context_page: ContextPage::Settings,
            about,
            nav: nav_model,
            key_binds: key_binds(),
            handler: flags.handler,
            config: flags.config.clone(),
            store: flags.store.clone(),
            content: Content::new(flags.store.clone(), flags.config),
            details: Details::new(flags.store),
            modifiers: Modifiers::empty(),
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
        };

        let mut tasks = vec![app.update(Message::Tasks(TasksAction::FetchLists))];

        if let Some(id) = app.core.main_window_id() {
            tasks.push(app.set_window_title(fl!("tasks"), id));
        }

        app.core.nav_bar_toggle_condensed();

        (app, app::Task::batch(tasks))
    }

    fn context_drawer(&self) -> Option<app::context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => app::context_drawer::about(
                &self.about,
                |url| Message::Open(url.to_string()),
                Message::ToggleContextDrawer,
            )
            .title(self.context_page.title()),
            ContextPage::Settings => {
                app::context_drawer::context_drawer(self.settings(), Message::ToggleContextDrawer)
                    .title(self.context_page.title())
            }
            ContextPage::TaskDetails => app::context_drawer::context_drawer(
                self.details.view().map(Message::Details),
                Message::ToggleContextDrawer,
            )
            .title(self.context_page.title()),
        })
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        let dialog_page = self.dialog_pages.front()?;
        let dialog = dialog_page.view(&self.dialog_text_input);
        Some(dialog.into())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![menu::menu_bar(&self.key_binds, &self.config)]
    }

    fn nav_context_menu(
        &self,
        id: widget::nav_bar::Id,
    ) -> Option<Vec<widget::menu::Tree<cosmic::Action<Self::Message>>>> {
        Some(cosmic::widget::menu::items(
            &HashMap::new(),
            vec![
                cosmic::widget::menu::Item::Button(
                    fl!("rename"),
                    Some(widget::icon::from_name("edit-symbolic").size(14).handle()),
                    NavMenuAction::Rename(id),
                ),
                cosmic::widget::menu::Item::Button(
                    fl!("icon"),
                    Some(
                        widget::icon::from_name("face-smile-big-symbolic")
                            .size(14)
                            .handle(),
                    ),
                    NavMenuAction::SetIcon(id),
                ),
                cosmic::widget::menu::Item::Button(
                    fl!("export"),
                    Some(widget::icon::from_name("share-symbolic").size(18).handle()),
                    NavMenuAction::Export(id),
                ),
                cosmic::widget::menu::Item::Button(
                    fl!("delete"),
                    Some(
                        widget::icon::from_name("user-trash-full-symbolic")
                            .size(14)
                            .handle(),
                    ),
                    NavMenuAction::Delete(id),
                ),
            ],
        ))
    }

    fn nav_model(&self) -> Option<&widget::segmented_button::SingleSelectModel> {
        Some(&self.nav)
    }

    fn on_escape(&mut self) -> app::Task<Self::Message> {
        if self.dialog_pages.pop_front().is_some() {
            return app::Task::none();
        }

        self.core.window.show_context = false;

        app::Task::none()
    }

    fn on_nav_select(&mut self, entity: Entity) -> app::Task<Self::Message> {
        let mut tasks = vec![];
        self.nav.activate(entity);
        let location_opt = self.nav.data::<List>(entity);

        if let Some(list) = location_opt {
            let message = Message::Content(content::Message::SetList(Some(list.clone())));
            let window_title = format!("{} - {}", list.name, fl!("tasks"));
            if let Some(window_id) = self.core.main_window_id() {
                tasks.push(self.set_window_title(window_title, window_id));
            }
            return self.update(message);
        }

        app::Task::batch(tasks)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let mut subscriptions = vec![
            cosmic::iced::event::listen_with(|event, _status, _window_id| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => {
                    Some(Message::Application(ApplicationAction::Key(modifiers, key)))
                }
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => Some(
                    Message::Application(ApplicationAction::Modifiers(modifiers)),
                ),
                _ => None,
            }),
            cosmic_config::config_subscription(
                TypeId::of::<ConfigSubscription>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|update: Update<ThemeMode>| {
                if !update.errors.is_empty() {
                    tracing::info!(
                        "errors loading config {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::Application(ApplicationAction::SystemThemeModeChange)
            }),
            cosmic_config::config_subscription::<_, cosmic_theme::ThemeMode>(
                TypeId::of::<ThemeSubscription>(),
                cosmic_theme::THEME_MODE_ID.into(),
                cosmic_theme::ThemeMode::version(),
            )
            .map(|update: Update<ThemeMode>| {
                if !update.errors.is_empty() {
                    tracing::info!(
                        "errors loading theme mode {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::Application(ApplicationAction::SystemThemeModeChange)
            }),
        ];

        subscriptions.push(self.content.subscription().map(Message::Content));

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Self::Message> {
        match message {
            Message::Open(url) => {
                if let Err(err) = open::that_detached(url) {
                    tracing::error!("{err}")
                }
            }
            Message::Content(message) => {
                if let Some(output) = self.content.update(message) {
                    match output {
                        content::Output::Focus(id) => return cosmic::widget::text_input::focus(id),
                        content::Output::OpenTaskDetails(task) => {
                            let tasks = vec![
                                cosmic::task::message(Message::Details(details::Message::SetTask(
                                    task.clone(),
                                    self.nav.active_data::<List>().map(|list| list.id),
                                ))),
                                cosmic::task::message(Message::ToggleContextPage(
                                    ContextPage::TaskDetails,
                                )),
                            ];
                            return app::Task::batch(tasks);
                        }
                        content::Output::ToggleHideCompleted(list) => {
                            if let Some(data) = self.nav.active_data_mut::<List>() {
                                data.hide_completed = list.hide_completed;
                            }
                        }
                    }
                }
            }
            Message::Details(message) => {
                if let Some(output) = self.details.update(message) {
                    match output {
                        details::Output::OpenCalendarDialog => {
                            return cosmic::task::message(Message::Dialog(DialogAction::Open(
                                DialogPage::Calendar(CalendarModel::now()),
                            )));
                        }
                        details::Output::RefreshTask(task) => {
                            return cosmic::task::message(Message::Content(
                                content::Message::RefreshTask(task.clone()),
                            ));
                        }
                    }
                }
            }
            Message::Tasks(action) => match action {
                TasksAction::FetchLists => match self.store.lists().load_all() {
                    Ok(lists) => {
                        return self.update(Message::Tasks(TasksAction::PopulateLists(lists)));
                    }
                    Err(err) => {
                        tracing::error!("Error fetching lists: {err}");
                    }
                },
                TasksAction::PopulateLists(lists) => {
                    for list in lists {
                        self.create_nav_item(&list);
                    }
                    let Some(entity) = self.nav.iter().next() else {
                        return app::Task::none();
                    };
                    self.nav.activate(entity);
                    return self.on_nav_select(entity);
                }
                TasksAction::AddList(list) => {
                    self.create_nav_item(&list);
                    let Some(entity) = self.nav.iter().last() else {
                        return app::Task::none();
                    };
                    return self.on_nav_select(entity);
                }
                TasksAction::DeleteList(entity) => {
                    let data = if let Some(entity) = entity {
                        self.nav.data::<List>(entity)
                    } else {
                        self.nav.active_data::<List>()
                    };
                    if let Some(list) = data {
                        if let Err(err) = self.store.lists().delete(list.id) {
                            tracing::error!("Error deleting list: {err}");
                        }

                        return cosmic::task::message(Message::Content(content::Message::SetList(
                            None,
                        )));
                    }
                    self.nav.remove(self.nav.active());
                }
            },
            Message::Application(action) => match action {
                ApplicationAction::AppTheme(theme) => {
                    if let Err(err) = self.config.set_app_theme(&self.handler, theme.into()) {
                        tracing::error!("{err}")
                    }
                }
                ApplicationAction::SystemThemeModeChange => {
                    return cosmic::command::set_theme(self.config.app_theme.theme());
                }
                ApplicationAction::Key(modifiers, key) => {
                    for (key_bind, action) in self.key_binds.clone().into_iter() {
                        if key_bind.matches(modifiers, &key) {
                            return cosmic::task::message(action.message());
                        }
                    }
                }
                ApplicationAction::Modifiers(modifiers) => {
                    self.modifiers = modifiers;
                }
                ApplicationAction::Focus(id) => {
                    return cosmic::task::message(Message::Application(ApplicationAction::Focus(
                        id,
                    )));
                }
            },
            Message::Menu(action) => match action {
                MenuAction::About => {
                    return cosmic::task::message(Message::ToggleContextPage(ContextPage::About));
                }
                MenuAction::Settings => {
                    return cosmic::task::message(Message::ToggleContextPage(
                        ContextPage::Settings,
                    ));
                }
                MenuAction::WindowClose => {
                    if let Some(window_id) = self.core.main_window_id() {
                        return cosmic::iced::window::close(window_id);
                    }
                }
                MenuAction::WindowNew => match env::current_exe() {
                    Ok(exe) => match process::Command::new(&exe).spawn() {
                        Ok(_) => {}
                        Err(err) => {
                            tracing::error!("failed to execute {exe:?}: {err}");
                        }
                    },
                    Err(err) => {
                        tracing::error!("failed to get current executable path: {err}");
                    }
                },
                MenuAction::NewList => {
                    return cosmic::task::message(Message::Dialog(DialogAction::Open(
                        DialogPage::New(String::new()),
                    )));
                }
                MenuAction::DeleteList => {
                    return cosmic::task::message(Message::Dialog(DialogAction::Open(
                        DialogPage::Delete(None),
                    )));
                }
                MenuAction::RenameList => {
                    return cosmic::task::message(Message::Dialog(DialogAction::Open(
                        DialogPage::Rename(None, String::new()),
                    )));
                }
                MenuAction::Icon => {
                    return cosmic::task::message(Message::Dialog(DialogAction::Open(
                        DialogPage::Icon(None, String::new(), String::new()),
                    )));
                }
                MenuAction::ToggleHideCompleted(completed) => {
                    if let Err(err) = self.config.set_hide_completed(&self.handler, completed) {
                        tracing::error!("{err}")
                    }
                    return cosmic::task::message(Message::Content(content::Message::SetConfig(
                        self.config.clone(),
                    )));
                }
                MenuAction::SortByNameAsc => {
                    return cosmic::task::message(Message::Content(content::Message::SetSort(
                        content::SortType::NameAsc,
                    )));
                }
                MenuAction::SortByNameDesc => {
                    return cosmic::task::message(Message::Content(content::Message::SetSort(
                        content::SortType::NameDesc,
                    )));
                }
                MenuAction::SortByDateAsc => {
                    return cosmic::task::message(Message::Content(content::Message::SetSort(
                        content::SortType::DateAsc,
                    )));
                }
                MenuAction::SortByDateDesc => {
                    return cosmic::task::message(Message::Content(content::Message::SetSort(
                        content::SortType::DateDesc,
                    )));
                }
            },
            Message::Dialog(action) => match action {
                DialogAction::Open(page) => {
                    match page {
                        DialogPage::Rename(entity, _) => {
                            let data = if let Some(entity) = entity {
                                self.nav.data::<List>(entity)
                            } else {
                                self.nav.active_data::<List>()
                            };
                            if let Some(list) = data {
                                self.dialog_pages
                                    .push_back(DialogPage::Rename(entity, list.name.clone()));
                            }
                        }
                        page => self.dialog_pages.push_back(page),
                    }
                    return cosmic::task::message(Message::Application(ApplicationAction::Focus(
                        self.dialog_text_input.clone(),
                    )));
                }
                DialogAction::Update(dialog_page) => {
                    self.dialog_pages[0] = dialog_page;
                }
                DialogAction::Close => {
                    self.dialog_pages.pop_front();
                }
                DialogAction::Complete => {
                    if let Some(message) = self.complete_dialog() {
                        return cosmic::task::message(message);
                    }
                }
                DialogAction::None => (),
            },
            Message::ToggleContextPage(page) => {
                if self.context_page == page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = page;
                    self.core.window.show_context = true;
                }
                return cosmic::task::message(Message::Content(content::Message::ContextMenuOpen(
                    self.core.window.show_context,
                )));
            }
            Message::NavMenu(action) => match action {
                NavMenuAction::Rename(entity) => {
                    return cosmic::task::message(Message::Dialog(DialogAction::Open(
                        DialogPage::Rename(Some(entity), String::new()),
                    )))
                }
                NavMenuAction::SetIcon(entity) => {
                    return cosmic::task::message(Message::Dialog(DialogAction::Open(
                        DialogPage::Icon(Some(entity), String::new(), String::new()),
                    )))
                }
                NavMenuAction::Export(entity) => {
                    if let Some(list) = self.nav.data::<List>(entity) {
                        match self.store.tasks(list.id).load_all() {
                            Ok(data) => {
                                let exported_markdown = Self::export_list(list, &data);
                                return cosmic::task::message(Message::Dialog(DialogAction::Open(
                                    DialogPage::Export(exported_markdown),
                                )));
                            }
                            Err(err) => {
                                tracing::error!("Error fetching tasks: {err}");
                            }
                        }
                    }
                }
                NavMenuAction::Delete(entity) => {
                    return cosmic::task::message(Message::Dialog(DialogAction::Open(
                        DialogPage::Delete(Some(entity)),
                    )));
                }
            },
            Message::ToggleContextDrawer => {
                self.core.window.show_context = !self.core.window.show_context;
                return cosmic::task::message(Message::Content(content::Message::ContextMenuOpen(
                    self.core.window.show_context,
                )));
            }
        }

        app::Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.content.view().map(Message::Content)
    }
}

impl AppModel {
    fn settings(&self) -> Element<'_, Message> {
        widget::scrollable(widget::settings::section().title(fl!("appearance")).add(
            widget::settings::item::item(
                fl!("theme"),
                widget::dropdown(
                    vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
                    Some(self.config.app_theme.into()),
                    |theme| Message::Application(ApplicationAction::AppTheme(theme)),
                ),
            ),
        ))
        .into()
    }

    fn create_nav_item(&mut self, list: &List) -> EntityMut<'_, SingleSelect> {
        let icon =
            widget::icon::from_name(list.icon.as_deref().unwrap_or("view-list-symbolic")).size(16);
        self.nav
            .insert()
            .text(list.name.clone())
            .icon(icon)
            .data(list.clone())
    }

    fn complete_dialog(&mut self) -> Option<Message> {
        if let Some(dialog_page) = self.dialog_pages.pop_front() {
            match dialog_page {
                DialogPage::New(name) => {
                    let list = List::new(&name);
                    match self.store.lists().save(&list) {
                        Ok(_) => {
                            return Some(Message::Tasks(TasksAction::AddList(list)));
                        }
                        Err(err) => {
                            tracing::error!("Error updating list: {err}");
                        }
                    }
                }
                DialogPage::Rename(entity, name) => {
                    let data = if let Some(entity) = entity {
                        self.nav.data_mut::<List>(entity)
                    } else {
                        self.nav.active_data_mut::<List>()
                    };

                    if let Some(list) = data {
                        match self
                            .store
                            .lists()
                            .update(list.id, |l| l.name = name.clone())
                        {
                            Ok(_) => {
                                list.name.clone_from(&name.clone());
                                let list = list.clone();
                                self.nav.text_set(self.nav.active(), name.clone());
                                return Some(Message::Content(content::Message::SetList(Some(
                                    list,
                                ))));
                            }
                            Err(err) => {
                                tracing::error!("Error updating list: {err}");
                            }
                        }
                    }
                }
                DialogPage::Delete(entity) => {
                    return Some(Message::Tasks(TasksAction::DeleteList(entity)));
                }
                DialogPage::Icon(entity, name, _) => {
                    let data = if let Some(entity) = entity {
                        self.nav.data::<List>(entity)
                    } else {
                        self.nav.active_data::<List>()
                    };
                    if let Some(list) = data {
                        let entity = self.nav.active();
                        self.nav.text_set(entity, list.name.clone());
                        self.nav.icon_set(
                            entity,
                            widget::icon::from_name(name.clone()).size(16).icon(),
                        );
                    }
                    if let Some(list) = self.nav.active_data_mut::<List>() {
                        list.icon = Some(name);
                        let list = list.clone();
                        if let Err(err) = self
                            .store
                            .lists()
                            .update(list.id, |l| l.icon = list.icon.clone())
                        {
                            tracing::error!("Error updating list: {err}");
                        }
                        return Some(Message::Content(content::Message::SetList(Some(list))));
                    }
                }
                DialogPage::Calendar(date) => {
                    self.details
                        .update(details::Message::SetDueDate(date.selected));
                }
                DialogPage::Export(content) => {
                    let Ok(mut clipboard) = ClipboardContext::new() else {
                        tracing::error!("Clipboard is not available");
                        return None;
                    };
                    if let Err(error) = clipboard.set_contents(content) {
                        tracing::error!("Error setting clipboard contents: {error}");
                    }
                }
            }
        }
        None
    }

    pub fn export_list(list: &List, tasks: &[crate::model::Task]) -> String {
        let markdown = list.markdown();
        let tasks_markdown: String = tasks.iter().map(Markdown::markdown).collect();
        format!("{markdown}\n{tasks_markdown}")
    }
}
