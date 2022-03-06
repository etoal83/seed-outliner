#![allow(clippy::wildcard_imports)]
// TODO: Remove
#![allow(dead_code, unused_variables)]

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

// TODO: Remove
impl Model {
    fn add_mock_data(mut self) -> Self {
        let (id_0, id_1) = (Uuid::new_v4(), Uuid::new_v4());

        self.outline.insert(id_0, Node {
            id: id_0,
            content: "First node.".to_owned(),
            children: Nodes::new(),
            folded: false,
        });

        self.outline.insert(id_1, Node {
            id: id_1,
            content: "Second node.".to_owned(),
            children: Nodes::new(),
            folded: false,
        });

        self
    }
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
