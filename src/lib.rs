#![allow(clippy::wildcard_imports)]
// TODO: Remove
#![allow(dead_code, unused_variables)]

use indexmap::IndexMap;
use indextree::{Arena, NodeId as Vertex};
use seed::{prelude::*, *};
use seed_styles::{*, px, rem};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

// ------ ------
//     Init
// ------ ------

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    let root_node_id = Uuid::new_v4();
    let root_node = Node {
        id: root_node_id,
        content: "".to_string(),
        folded: false,
    };
    let mut arena = Arena::new();
    let root = arena.new_node(root_node.clone());

    // TODO: Remove
    let mut nodes = Nodes::new();
    nodes.insert(root_node_id, root_node);
    
    orders.stream(streams::document_event(Ev::SelectionChange, |_| {
        Msg::CaretPositionChanged
    }));
    
    Model {
        tree: arena,
        root: root,
        editing_node: None,
        // TODO: Remove
        nodes: nodes,
    }.add_mock_data()
}

// ------ ------
//     Model
// ------ ------

struct Model {
    tree: Arena<Node>,
    root: Vertex,
    editing_node: Option<EditingNode>,
    // TODO: Remove field
    nodes: Nodes,
}

// TODO: Remove
type Nodes = IndexMap<Uuid, Node>;

#[derive(Clone)]
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
    caret_position: u32,
}

// TODO: Remove
impl Model {
    fn add_mock_data(mut self) -> Self {
        let (id_0, id_1, id_2, id_0_0, id_2_0) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
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
        let third_node = Node {
            id: id_2,
            content: "Third node.".to_owned(),
            folded: false,
        };
        let first_child_node = Node {
            id: id_0_0,
            content: "First child node.".to_owned(),
            folded: false,            
        };
        let third_child_node = Node {
            id: id_0_0,
            content: "Third child node.".to_owned(),
            folded: false,
        };
        // TODO: Remove
        self.nodes.insert(id_0, first_node.clone());
        self.nodes.insert(id_1, second_node.clone());
        self.nodes.insert(id_2, third_node.clone());
        self.nodes.insert(id_0_0, first_child_node.clone());
        self.nodes.insert(id_2_0, third_child_node.clone());

        let first_node = self.tree.new_node(first_node);
        let second_node = self.tree.new_node(second_node);
        let third_node = self.tree.new_node(third_node);
        let first_child_node = self.tree.new_node(first_child_node);
        let third_child_node = self.tree.new_node(third_child_node);
        self.root.append(first_node, &mut self.tree);
        self.root.append(second_node, &mut self.tree);
        self.root.append(third_node, &mut self.tree);
        first_node.append(first_child_node, &mut self.tree);
        third_node.append(third_child_node, &mut self.tree);

        self
    }
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    StartEditingNodeContent(Option<Vertex>),
    EditingNodeContentChanged(String),
    SaveEditedNodeContent,
    InsertNewNode,
    DeleteNodeBackward,
    RemoveNode(Vertex),
    CaretPositionChanged,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::StartEditingNodeContent(Some(vertex)) => {
            if let Some(node) = model.tree.get(vertex) {
                let node = node.get();
                let content_element = ElRef::new();
                let selection = document().get_selection().expect("get selection").unwrap();
                let caret_position = selection.focus_offset();

                model.editing_node = Some(EditingNode {
                    id: node.id,
                    content: node.content.clone(),
                    content_element: content_element.clone(),
                    vertex,
                    caret_position: caret_position,
                });

                orders.after_next_render(move |_| {
                    let content_element = content_element.get().expect("content_element");

                    content_element
                        .focus()
                        .expect("focus content_element");
                });
            }
        },
        Msg::StartEditingNodeContent(None) => {
            log!("StartEditingNodeContent: None");
            model.editing_node = None;
        },
        Msg::EditingNodeContentChanged(content) => {
            log!("EditingNodeContentChanged", content);
            if let Some(editing_node) = &mut model.editing_node {
                editing_node.content = content;
            }
        },
        Msg::SaveEditedNodeContent => {
            log!("SaveEditedNodeContent");
            if let Some(editing_node) = model.editing_node.take() {
                let mut node = model.tree.get_mut(editing_node.vertex).expect("vertex exists").get_mut();
                node.content = editing_node.content.to_owned();
            }
        },
        Msg::InsertNewNode => {
            log!("InsertNewNode");
            if let Some(editing_node) = &mut model.editing_node {
                let selection = document().get_selection().expect("get selection").unwrap();
                let caret_position = selection.focus_offset();
                let s = UnicodeSegmentation::graphemes(editing_node.content.as_str(), true).collect::<Vec<&str>>();
                let (left, right) = s.split_at(caret_position as usize);
                orders.send_msg(Msg::EditingNodeContentChanged(left.join("").to_owned()));
                orders.send_msg(Msg::SaveEditedNodeContent);

                let new_id = Uuid::new_v4();
                
                let new_node = model.tree.new_node(Node {
                    id: new_id,
                    content: right.join("").to_owned(),
                    folded: false,
                });
                editing_node.vertex.insert_after(new_node, &mut model.tree);
                orders.send_msg(Msg::StartEditingNodeContent(Some(new_node)));
            }
        },
        Msg::DeleteNodeBackward => {
            log!("DeleteNodeBackward");
            let mut destination = None;
            let dest= &mut destination;

            if let Some(editing_node) = &model.editing_node {
                let has_children = editing_node.vertex.children(&model.tree).next().is_some();
                let previous_sibling = editing_node.vertex.preceding_siblings(&model.tree).skip(1).next();
                let parent = editing_node.vertex.ancestors(&model.tree).skip(1).next();
                *dest = match (has_children, previous_sibling, parent) {
                    (false, Some(sibling), _) => sibling.descendants(&model.tree).last(),
                    (false, None, Some(parent)) => if parent != model.root { Some(parent) } else { None },
                    (true, Some(sibling), _) => if sibling.children(&model.tree).next().is_none() { Some(sibling) } else { None },
                    (_, _, _) => None,
                };
            };

            if let Some(vertex) = destination {
                if let Some(editing_node) = model.editing_node.take() {
                    let taken_content = editing_node.content.to_owned();
                    orders.send_msg(Msg::RemoveNode(editing_node.vertex));
                    
                    let node = model.tree.get_mut(vertex).unwrap().get_mut();
                    let caret_position_dest = UnicodeSegmentation::graphemes(node.content.as_str(), true).collect::<Vec<&str>>().len() as u32;
                    node.content = format!("{}{}", node.content.to_owned(), taken_content);
                    let content_element = ElRef::new();

                    model.editing_node = Some(EditingNode {
                        id: node.id,
                        content: node.content.clone(),
                        content_element: content_element.clone(),
                        vertex,
                        caret_position: 0,
                    });

                    orders.after_next_render(move |_| {
                        let content_element = content_element.get().expect("content_element");
                        content_element
                            .focus()
                            .expect("focus content_element");

                        let selection = document().get_selection().expect("get selection").unwrap();
                        let range = document().create_range().expect("create range");
                        range
                            .set_start(&content_element.child_nodes().get(0).unwrap(), caret_position_dest)
                            .expect("Range: set start");
                        range.collapse();    
                        selection
                            .remove_all_ranges()
                            .expect("Selection: remove all ranges");
                        selection
                            .add_range(&range)
                            .expect("Selection: add range")
                    });
                }
            }
        },
        Msg::RemoveNode(vertex) => {
            log!("RemoveNode");
            let node = model.tree.get(vertex).unwrap().get();
            vertex.remove(&mut model.tree);
        },
        Msg::CaretPositionChanged => {
            if let Some(editing_node) = &mut model.editing_node {
                let selection = document().get_selection().expect("get selection").unwrap();
                let caret_position = selection.focus_offset();
                editing_node.caret_position = caret_position;
                log!("Msg::CaretPositionChanged", caret_position);
            }
        }
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> seed::virtual_dom::Node<Msg> {
    div![
        // TODO: Remove argument `nodes`
        view_nodes(&model.nodes, &model.tree, &model.root, model.editing_node.as_ref()),
    ]
}

// TODO: Remove argument `nodes`
fn view_nodes(nodes: &Nodes, tree: &Arena<Node>, current_vertex: &Vertex, editing_node: Option<&EditingNode>) -> Vec<seed::virtual_dom::Node<Msg>> {
    current_vertex.children(tree).map(|vertex| {
        let node = tree.get(vertex).unwrap().get();
        let is_editing = Some(node.id) == editing_node.map(|editing_node| editing_node.id);
        let is_deletable = editing_node.map_or(false, |editing_node| editing_node.caret_position == 0);

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
                    ev(Ev::Focus, move |_| Msg::StartEditingNodeContent(Some(vertex))),
                    ev(Ev::Blur, |_| Msg::SaveEditedNodeContent),
                    ev(Ev::Input, |event| {
                        let target = event.current_target().unwrap();
                        let content = target.dyn_ref::<web_sys::HtmlElement>().unwrap().text_content().unwrap();
                        Msg::EditingNodeContentChanged(content)
                    }),
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && keyboard_event.key_code() != 229 && keyboard_event.key().as_str() == "Enter" => {
                            keyboard_event.prevent_default();
                            Msg::InsertNewNode
                        })
                    }),
                    keyboard_ev(Ev::KeyDown, move |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && keyboard_event.key_code() != 229 && is_deletable && keyboard_event.key().as_str() == "Backspace" => {
                            Msg::DeleteNodeBackward
                        })
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
