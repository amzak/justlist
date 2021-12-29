use serde::{Deserialize, Serialize};

use cursive::traits::*;
use cursive::view::SizeConstraint;
use cursive::views::{EditView, LinearLayout, SelectView, TextView};
use cursive::Cursive;

#[derive(Serialize, Deserialize)]
struct ListItem {
    label: String,
    group_index: i32,
    param: String,
}

#[derive(Serialize, Deserialize)]
struct List {
    items: Vec<ListItem>,
    groups: Vec<String>,
    command_template: String,
}

fn sample() -> serde_json::Result<List> {
    let json = r#"
    {
        "command_template": "qqq",
        "groups": [
            "group 1",
            "group 2"
        ],
        "items": [
            {
                "label": "item 1",
                "group_index": 0,
                "param": "xxx"
            },
            {
                "label": "item 2",
                "group_index": 0,
                "param": "yyy"
            }
        ]
    }"#;

    let list: List = serde_json::from_str(json)?;
    Ok(list)
}

fn main() {
    let mut siv = cursive::default();

    siv.add_global_callback('q', Cursive::quit);

    let model: List = sample().unwrap();

    let iter = model.items.into_iter();

    let input = EditView::new()
        .on_submit(on_input)
        .with_name("input")
        .fixed_width(10);
    let select = SelectView::new()
        .with_all(iter.map(|item| (item.label.clone(), item)))
        .on_submit(on_select)
        .with_name("select")
        .resized(SizeConstraint::Full, SizeConstraint::Full);

    siv.add_fullscreen_layer(
        LinearLayout::vertical()
            .child(select)
            .child(TextView::new("q Exit")),
    );

    siv.run();
}

fn on_select(s: &mut Cursive, item: &ListItem) {}

fn on_input(s: &mut Cursive, input: &str) {}
