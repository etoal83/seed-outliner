#![allow(clippy::wildcard_imports)]
// TODO: Remove
#![allow(dead_code, unused_variables)]

use indexmap::IndexMap;
use indextree::{Arena, NodeId as Vertex};
use seed::{prelude::*, *};
use seed_styles::{*, px, rem};
use uuid::Uuid;

// ------ ------
//     Init
// ------ ------

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let root_node_id = Uuid::new_v4();
    let mut arena = Arena::new();
    let root = arena.new_node(root_node_id);

    let root_node = Node {
        id: root_node_id,
        content: root.to_string(),
        folded: false,
    };
    let mut nodes = Nodes::new();
    nodes.insert(root_node_id, root_node);
    
    Model {
        nodes: nodes,
        tree: arena,
        root: root,
        editing_node: None,
    }.add_mock_data()
}

// ------ ------
//     Model
// ------ ------

struct Model {
    nodes: Nodes,
    tree: Arena<Uuid>,
    root: Vertex,
    editing_node: Option<EditingNode>,
}

type Nodes = IndexMap<Uuid, Node>;

struct Node {
    id: Uuid,
    content: String,
    folded: bool,
}

#[derive(Debug)]
struct EditingNode {
    id: Uuid,
    content: String,
    content_element: ElRef<web_sys::HtmlElement>,
    vertex: Vertex,
}

// TODO: Remove
impl Model {
    fn add_mock_data(mut self) -> Self {
        let (id_0, id_1, id_0_0) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let first_node = Node {
            id: id_0,
            content: "First node.".to_owned(),
            folded: false,
        };
        let second_node = Node {
            id: id_1,
            content: "Second node.".to_owned(),
            folded: false,
        };
        let first_child_node = Node {
            id: id_0_0,
            content: "First child node.".to_owned(),
            folded: false,            
        };
        self.nodes.insert(id_0, first_node);
        self.nodes.insert(id_1, second_node);
        self.nodes.insert(id_0_0, first_child_node);

        let first_node = self.tree.new_node(id_0);
        let second_node = self.tree.new_node(id_1);
        let first_child_node = self.tree.new_node(id_0_0);
        self.root.append(first_node, &mut self.tree);
        self.root.append(second_node, &mut self.tree);
        first_node.append(first_child_node, &mut self.tree);

        self
    }
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    EditNodeContent(Option<Vertex>),
    EditingNodeContentChanged(String),
    InsertNewNode(String),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::EditNodeContent(Some(vertex)) => {
            log!("EditNodeContent: ", vertex);
            if let Some(node) = model.tree.get(vertex) {
                let id = *node.get();
                let node = model.nodes.get(&id).unwrap();
                let content_element = ElRef::new();

                model.editing_node = Some(EditingNode {
                    id,
                    content: node.content.clone(),
                    content_element: content_element.clone(),
                    vertex,
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
        Msg::InsertNewNode(content) => {
            log!("InsertNewNode", content);
        }
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> seed::virtual_dom::Node<Msg> {
    div![
        view_nodes(&model.nodes, &model.tree, &model.root, model.editing_node.as_ref()),
    ]
}

fn view_nodes(nodes: &Nodes, tree: &Arena<Uuid>, current_vertex: &Vertex, editing_node: Option<&EditingNode>) -> Vec<seed::virtual_dom::Node<Msg>> {
    current_vertex.children(tree).map(|vertex| {
        let id = *tree.get(vertex).unwrap().get();
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
                    ev(Ev::Click, move |_| Msg::EditNodeContent(Some(vertex))),
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
                IF!(vertex.children(tree).peekable().peek().is_some() => view_nodes(nodes, tree, &vertex, editing_node)),
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
