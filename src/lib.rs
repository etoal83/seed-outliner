#![allow(clippy::wildcard_imports)]
// TODO: Remove
#![allow(dead_code, unused_variables)]

use indexmap::IndexMap;
use seed::{prelude::*, *};
use seed_styles::{*, px, rem};
use uuid::Uuid;

// ------ ------
//     Init
// ------ ------

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        nodes: Nodes::new(),
        current_children: Vec::new(),
        editing_node: None,
    }.add_mock_data()
}

// ------ ------
//     Model
// ------ ------

struct Model {
    nodes: Nodes,
    current_children: Vec<Uuid>,
    editing_node: Option<EditingNode>,
}

type Nodes = IndexMap<Uuid, Node>;

struct Node {
    id: Uuid,
    content: String,
    children: Vec<Uuid>,
    folded: bool,
}

#[derive(Debug)]
struct EditingNode {
    id: Uuid,
    content: String,
    content_element: ElRef<web_sys::HtmlElement>,
}

// TODO: Remove
impl Model {
    fn add_mock_data(mut self) -> Self {
        let (id_0, id_1, id_0_0) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());

        self.nodes.insert(id_0, Node {
            id: id_0,
            content: "First node.".to_owned(),
            children: vec![id_0_0],
            folded: false,
        });

        self.nodes.insert(id_1, Node {
            id: id_1,
            content: "Second node.".to_owned(),
            children: Vec::new(),
            folded: false,
        });

        self.nodes.insert(id_0_0, Node {
            id: id_1,
            content: "First child node.".to_owned(),
            children: Vec::new(),
            folded: false,
        });

        self.current_children.extend_from_slice(&[id_0, id_1]);

        self
    }
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    EditNodeContent(Option<Uuid>),
    EditingNodeContentChanged(String),
    InsertNewNode,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::EditNodeContent(Some(id)) => {
            log!("EditNodeContent: ", id);
            if let Some(node) = model.nodes.get(&id) {
                let content_element = ElRef::new();

                model.editing_node = Some(EditingNode {
                    id,
                    content: node.content.clone(),
                    content_element: content_element.clone(),
                });

                orders.after_next_render(move |_| {
                    let content_element = content_element.get().expect("content_element");

                    content_element
                        .focus()
                        .expect("focus content_element");
                });
            }
        },
        Msg::EditNodeContent(None) => {
            log!("EditNodeContent: None");
            model.editing_node = None;
        },
        Msg::EditingNodeContentChanged(content) => {
            log!("EditingNodeContentChanged", content);
            if let Some(editing_node) = &mut model.editing_node {
                editing_node.content = content;
            }
        },
        Msg::InsertNewNode => {
            log!("InsertNewNode");

        }
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> seed::virtual_dom::Node<Msg> {
    div![
        view_nodes(&model.nodes, &model.current_children, model.editing_node.as_ref()),
    ]
}

fn view_nodes(nodes: &Nodes, children: &Vec<Uuid>, editing_node: Option<&EditingNode>) -> Vec<seed::virtual_dom::Node<Msg>> {
    children.to_vec().into_iter().map(|id| {
        let node = nodes.get(&id).unwrap();
        let is_editing = Some(id) == editing_node.map(|editing_node| editing_node.id);

        div![
            C!["node"],
            div![
                C!["node-self"],
                el_key(&node.id),
                s().padding_y(rem(0.2)),
                a![
                    C!["node-bullet"],
                    s().display(CssDisplay::InlineBlock),
                    span![
                        C!["material-icons"],
                        s().font_size(rem(0.7)),
                        "fiber_manual_record"
                    ],
                    attrs!{At::TabIndex => -1},
                ],
                div![
                    C!["node-content"],
                    IF!(is_editing => {
                        let editing_node = editing_node.unwrap();
                        el_ref(&editing_node.content_element)
                    }),
                    s().display(CssDisplay::InlineBlock)
                        .margin_left(px(8))
                        .font_size(rem(1)),
                    attrs!{
                        At::ContentEditable => true,
                    },
                    &node.content,
                    ev(Ev::Click, move |_| Msg::EditNodeContent(Some(id))),
                    ev(Ev::Input, |event| {
                        let target = event.current_target().unwrap();
                        let content = target.dyn_ref::<web_sys::HtmlElement>().unwrap().text_content().unwrap();
                        Msg::EditingNodeContentChanged(content)
                    }),
                ],
            ],
            div![
                C!["node-children"],
                s().margin_left(px(10))
                    .border_left(CssBorderLeft::Border(CssBorderWidth::Length(px(1)), CssBorderStyle::Solid, CssColor::Rgba(0., 0., 0., 0.4)))
                    .padding_left(px(20)),
                IF!(not(&node.children.is_empty()) => view_nodes(nodes, &node.children, editing_node)),
            ]
        ]
    }).collect()
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
