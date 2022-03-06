#![allow(clippy::wildcard_imports)]
use indexmap::IndexMap;
use seed::{prelude::*, *};
use uuid::Uuid;

// ------ ------
//     Init
// ------ ------

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        outline: Nodes::new(),
    }
}

// ------ ------
//     Model
// ------ ------

struct Model {
    outline: Nodes
}

type Nodes = IndexMap<Uuid, Node>;

struct Node {
    id: Uuid,
    content: String,
    children: Nodes,
    folded: bool,
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    NodeContentChanged(String),
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::NodeContentChanged(content) => {
            log!("NodeContentChanged", content);
        }
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> seed::virtual_dom::Node<Msg> {
    div![
        "I'm a placeholder",
    ]
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
