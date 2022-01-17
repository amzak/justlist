#![deny(elided_lifetimes_in_paths)]

#[macro_use]
extern crate lazy_static;

use serde::{Deserialize, Serialize};
use std::fmt::Display;

use cursive::event::{Event, EventTrigger};
use cursive::event::Key;
use cursive::theme::Effect;
use cursive::theme::Style;
use cursive::traits::*;
use cursive::view::SizeConstraint;
use cursive::views::{DummyView, EditView, LinearLayout, SelectView, TextContent, TextView};
use cursive::Cursive;

use std::fmt;

#[derive(Serialize, Deserialize)]
struct ListItem {
    label: String,
    param: String,
}

impl fmt::Display for ListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

#[derive(Serialize, Deserialize)]
struct Group {
    label: String,
    items: Vec<ListItem>,
}

#[derive(Serialize, Deserialize)]
struct List {
    groups: Vec<Group>,
    command_template: String,
}

struct State {
    selected_group: usize,
    groups: Vec<Group>,
    command_template: String,
}

lazy_static! {
    static ref default_style: Style = Style::default();
    static ref selected_style: Style = Style::from(Effect::Reverse);
}

fn sample<'a>() -> serde_json::Result<List> {
    let json: &'a str = r#"
    {
        "command_template": "qqq",
        "groups": [
            {
                "label": "group 1",
                "items": [
                {
                    "label": "item 1",
                    "param": "xxx"
                },
                {
                    "label": "item 2",
                    "param": "yyy"
                }]
            },
            {
                "label": "group 2",
                "items": [
                {
                    "label": "item 3",
                    "param": "qqq"
                },
                {
                    "label": "item 4",
                    "param": "www"
                }]
            }
        ]
    }"#;

    let list: List = serde_json::from_str(json)?;
    Ok(list)
}

fn handle_left(s: &mut Cursive) {
    move_selected_group(s, -1);
}

fn move_selected_group(s: &mut Cursive, direction: i8) {
    let mut state: State = 
    s.take_user_data().unwrap();

    let old_selected_group = state.selected_group;
    let mut new_selected_group: i8 = state.selected_group as i8 + direction;
    if new_selected_group == state.groups.len() as i8 || new_selected_group < 0 {
        new_selected_group = 0;
    }

    let old_selected_name: &str = &state.groups[old_selected_group].label;
    let new_selected_name: &str = &state.groups[new_selected_group as usize].label;

    s.call_on_name(old_selected_name, |view: &mut TextView| {
        view.set_style(*default_style)
    });

    s.call_on_name(new_selected_name, |view: &mut TextView| {
        view.set_style(*selected_style)
    });

    state.selected_group = new_selected_group as usize;

    s.set_user_data(state);
}

fn handle_right(s: &mut Cursive) {
    move_selected_group(s, 1);
}

fn get_select_items(source: &Vec<ListItem>) -> Vec<(String, ListItem)> {
    source.into_iter().map(|x| (
        x.to_string(), 
        ListItem {label: x.label.clone(), param: x.param.clone()})
    ).collect()
}

fn main() {
    cursive::logger::init();

    let mut siv = cursive::default();
    siv.load_toml(include_str!("theme.toml")).unwrap();

    siv.add_global_callback('q', Cursive::quit);
    siv.add_global_callback('~', Cursive::toggle_debug_console);
    siv.add_global_callback(Key::Right, |s| handle_right(s));
    siv.add_global_callback(Key::Left, |s| handle_left(s));

    siv.set_autohide_menu(false);

    let input = EditView::new()
        .on_submit(on_input)
        .with_name("input")
        .fixed_width(10);

    let List {
        groups,
        command_template,
    } = sample().unwrap();

    let mut groups_view = LinearLayout::horizontal();

    for i in 0..groups.len() {
        let group_label = groups[i].label.clone();
        let content = TextContent::new(group_label.clone());
        let style = if i == 0 {
            *selected_style
        } else {
            *default_style
        };
        let text = TextView::new_with_content(content)
            .style(style)
            .with_name(group_label);
        groups_view.add_child(text);
        groups_view.add_child(DummyView {});
    }

    let items = &groups[0].items;

    let select_items = get_select_items(items);

    let select_view = SelectView::new()
        .with_all(select_items)
        .selected(0)
        .on_submit(on_select)
        .with_name("select")
        .resized(SizeConstraint::Full, SizeConstraint::Full);

    siv.add_fullscreen_layer(
        LinearLayout::vertical()
            .child(input)
            .child(groups_view)
            .child(select_view)
            .child(TextView::new("q Exit")),
    );

    let state = State {
        selected_group: 0,
        groups: groups,
        command_template: command_template,
    };
    siv.set_user_data(state);
    siv.run();
}

fn on_select(s: &mut Cursive, item: &ListItem) {}

fn on_input(s: &mut Cursive, input: &str) {}
