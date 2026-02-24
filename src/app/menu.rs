// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use cosmic::{
    Element, widget::{self, menu::{Item, ItemHeight, ItemWidth, MenuBar, Tree, items, key_bind::KeyBind, root}}
};

use crate::{
    app::{Action, Message},
    fl,
};

use crate::core::config::TasksConfig;

pub fn menu_bar<'a>(
    key_binds: &HashMap<KeyBind, Action>,
    config: &TasksConfig,
) -> Element<'a, Message> {
    MenuBar::new(vec![
        Tree::with_children(
            Element::from(root(fl!("file"))),
            items(
                key_binds,
                vec![
                    Item::Button(
                        fl!("new-window"),
                        Some(widget::icon::from_name("tabs-stack-symbolic").size(14).handle()),
                        Action::WindowNew,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("new-list"),
                        Some(widget::icon::from_name("plus-square-filled-symbolic").size(14).handle()),
                        Action::NewList,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("quit"),
                        Some(widget::icon::from_name("cross-small-square-filled-symbolic").size(14).handle()),
                        Action::WindowClose,
                    ),
                ],
            ),
        ),
        Tree::with_children(
            Element::from(root(fl!("edit"))),
            items(
                key_binds,
                vec![
                    Item::Button(
                        fl!("rename"),
                        Some(widget::icon::from_name("edit-symbolic").size(14).handle()),
                        Action::RenameList,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("icon"),
                        Some(widget::icon::from_name("face-smile-big-symbolic").size(14).handle()),
                        Action::Icon,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("delete"),
                        Some(widget::icon::from_name("user-trash-full-symbolic").size(14).handle()),
                        Action::DeleteList,
                    ),
                ],
            ),
        ),
        Tree::with_children(
            Element::from(root(fl!("view"))),
            items(
                key_binds,
                vec![
                    Item::Button(
                        fl!("menu-settings"),
                        Some(widget::icon::from_name("settings-symbolic").size(14).handle()),
                        Action::Settings,
                    ),
                    Item::Divider,
                    Item::CheckBox(
                        fl!("hide-completed"),
                        None,
                        config.hide_completed,
                        Action::ToggleHideCompleted(!config.hide_completed),
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("menu-about"),
                        Some(widget::icon::from_name("info-outline-symbolic").size(14).handle()),
                        Action::About,
                    ),
                ],
            ),
        ),
        Tree::with_children(
            Element::from(root(fl!("sort"))),
            items(
                key_binds,
                vec![
                    Item::Button(fl!("sort-name-asc"), None, Action::SortByNameAsc),
                    Item::Button(fl!("sort-name-desc"), None, Action::SortByNameDesc),
                    Item::Button(fl!("sort-date-asc"), None, Action::SortByDateAsc),
                    Item::Button(fl!("sort-date-desc"), None, Action::SortByDateDesc),
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(260))
    .spacing(4.0)
    .into()
}
