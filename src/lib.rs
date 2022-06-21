#![allow(clippy::wildcard_imports)]
// TODO: Remove
#![allow(dead_code, unused_variables)]

use generational_indextree::{Arena, NodeId as Vertex};
use seed::{prelude::*, *};
use seed_styles::{*, pc, px, rem};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

const TREE_STORAGE_KEY: &str = "seed-outliner-tree";

// ------ ------
//     Init
// ------ ------

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    let mut tree: Arena<Node> = LocalStorage::get(TREE_STORAGE_KEY).unwrap_or(Arena::new());

    let root = if tree.is_empty() {
        let root = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "🌱 Seed Outliner".to_string(),
            folded: false,
        });
        let node_welcome = Node {
            id: Uuid::new_v4(),
            content: "Welcome to Seed-Outliner!".to_owned(),
            folded: false,
        };
        let node_guide_start_edit = Node {
            id: Uuid::new_v4(),
            content: "You can click here to start editing contents.".to_owned(),
            folded: false,
        };
        let node_guide_indent = Node {
            id: Uuid::new_v4(),
            content: "You can indent items by pressing \"Tab\" key.".to_owned(),
            folded: false,
        };
        let node_guide_unindent = Node {
            id: Uuid::new_v4(),
            content: "...and unindent by pressing \"Shift + Tab\" key.".to_owned(),
            folded: false,            
        };

        let node_welcome = tree.new_node(node_welcome);
        let node_guide_start_edit = tree.new_node(node_guide_start_edit);
        let node_guide_indent = tree.new_node(node_guide_indent);
        let node_guide_unindent = tree.new_node(node_guide_unindent);
        root.append(node_welcome, &mut tree);
        root.append(node_guide_start_edit, &mut tree);
        root.append(node_guide_indent, &mut tree);
        node_guide_indent.append(node_guide_unindent, &mut tree);

        root
    } else {
        // TODO: Ensure to get root node
        tree.iter_pairs().next().unwrap().0
    };
    
    orders.stream(streams::document_event(Ev::SelectionChange, |_| {
        Msg::CaretPositionChanged
    }));
    
    Model {
        tree: tree,
        root: root,
        editing_node: None,
    }
}

// ------ ------
//     Model
// ------ ------

struct Model {
    tree: Arena<Node>,
    root: Vertex,
    editing_node: Option<EditingNode>,
}

#[derive(Debug, Deserialize, Serialize)]
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
    IndentNode,
    OutdentNode,
    MoveNodeUp,
    MoveNodeDown,
    FoldChildren(Vertex),
    UnfoldChildren(Vertex),
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
        },
        Msg::IndentNode => {
            let editing_node = match &mut model.editing_node {
                Some(node) => node,
                None => return,
            };

            if let Some(previous_sibling) = model.tree[editing_node.vertex].previous_sibling() {
                orders.send_msg(Msg::SaveEditedNodeContent);
                previous_sibling.append(editing_node.vertex, &mut model.tree);
                orders.send_msg(Msg::StartEditingNodeContent(Some(editing_node.vertex)));
            }
        },
        Msg::OutdentNode => {
            let editing_node = match &model.editing_node {
                Some(node) => node,
                None => return,
            };

            if let Some(parent) = model.tree[editing_node.vertex].parent() {
                if parent == model.root { return };
                orders.send_msg(Msg::SaveEditedNodeContent);

                let following_siblings = editing_node.vertex.following_siblings(&model.tree).skip(1).collect::<Vec<_>>();
                for node in following_siblings {
                    editing_node.vertex.append(node, &mut model.tree);
                }

                parent.insert_after(editing_node.vertex, &mut model.tree);
                orders.send_msg(Msg::StartEditingNodeContent(Some(editing_node.vertex)));
            }
        },
        Msg::MoveNodeUp => {
            log!("MoveNodeUp");
            let editing_node = match &mut model.editing_node {
                Some(node) => node,
                None => return,
            };

            if let Some(previous_sibling) = model.tree[editing_node.vertex].previous_sibling() {
                orders.send_msg(Msg::SaveEditedNodeContent);
                previous_sibling.insert_before(editing_node.vertex, &mut model.tree);
                orders.send_msg(Msg::StartEditingNodeContent(Some(editing_node.vertex)));
            } else if let Some(parent) = model.tree[editing_node.vertex].parent() {
                if let Some(parents_previous_sibling) = model.tree[parent].previous_sibling() {
                    orders.send_msg(Msg::SaveEditedNodeContent);
                    parents_previous_sibling.append(editing_node.vertex, &mut model.tree);
                    orders.send_msg(Msg::StartEditingNodeContent(Some(editing_node.vertex)));    
                }
            }
        },
        Msg::MoveNodeDown => {
            log!("MoveNodeDown");
            let editing_node = match &mut model.editing_node {
                Some(node) => node,
                None => return,
            };

            if let Some(next_sibling) = model.tree[editing_node.vertex].next_sibling() {
                orders.send_msg(Msg::SaveEditedNodeContent);
                next_sibling.insert_after(editing_node.vertex, &mut model.tree);
                orders.send_msg(Msg::StartEditingNodeContent(Some(editing_node.vertex)));
            } else if let Some(parent) = model.tree[editing_node.vertex].parent() {
                if let Some(parents_next_sibling) = model.tree[parent].next_sibling() {
                    orders.send_msg(Msg::SaveEditedNodeContent);
                    parents_next_sibling.prepend(editing_node.vertex, &mut model.tree);
                    orders.send_msg(Msg::StartEditingNodeContent(Some(editing_node.vertex)));    
                }
            }
        },
        Msg::FoldChildren(vertex) => {
            log!("FoldChildren");
            let node = model.tree.get_mut(vertex).unwrap().get_mut();
            node.folded = true;
        },
        Msg::UnfoldChildren(vertex) => {
            log!("UnfoldChildren");
            let node = model.tree.get_mut(vertex).unwrap().get_mut();
            node.folded = false;
        },
    }
    LocalStorage::insert(TREE_STORAGE_KEY, &model.tree).expect("node tree saved");
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Vec<seed::virtual_dom::Node<Msg>> {
    vec![
        h1![&model.tree[model.root].get().content],
        div![
            view_nodes(&model.tree, &model.root, model.editing_node.as_ref()),
        ],
    ]
}

fn view_nodes(tree: &Arena<Node>, current_vertex: &Vertex, editing_node: Option<&EditingNode>) -> Vec<seed::virtual_dom::Node<Msg>> {
    current_vertex.children(tree).map(|vertex| {
        let node = tree.get(vertex).unwrap().get();
        let is_editing = Some(node.id) == editing_node.map(|editing_node| editing_node.id);
        let is_deletable = editing_node.map_or(false, |editing_node| editing_node.caret_position == 0);

        div![
            C!["node"],
            div![
                C!["node-self"],
                el_key(&node.id),
                s().padding_y(rem(0.2))
                    .padding_x(px(5)),
                a![
                    C!["node-bullet"],
                    s().display(CssDisplay::InlineBlock)
                        .text_align(CssTextAlign::Center)
                        .vertical_align(CssVerticalAlign::Top)
                        .width(px(12)),
                    attrs!{At::TabIndex => -1},
                    IF!(!node.folded => span![
                        C!["material-icons"],
                        s().font_size(rem(0.7)),
                        "fiber_manual_record",
                        IF!(vertex.children(tree).next().is_some() => ev(Ev::Click, move |_| Msg::FoldChildren(vertex)))
                    ]),
                    IF!(node.folded => span![
                        C!["material-icons"],
                        s().font_size(rem(1.0)),
                        "play_arrow",
                        ev(Ev::Click, move |_| Msg::UnfoldChildren(vertex))
                    ]),
                ],
                div![
                    C!["node-content"],
                    IF!(is_editing => {
                        let editing_node = editing_node.unwrap();
                        el_ref(&editing_node.content_element)
                    }),
                    s().display(CssDisplay::InlineBlock)
                        .margin_left(px(8))
                        .font_size(rem(1))
                        .width(CssWidth::Percentage(pc(90))),
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
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && !keyboard_event.shift_key() && keyboard_event.key().as_str() == "Tab" => {
                            keyboard_event.prevent_default();
                            Msg::IndentNode
                        })
                    }),
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && keyboard_event.shift_key() && keyboard_event.key().as_str() == "Tab" => {
                            keyboard_event.prevent_default();
                            Msg::OutdentNode
                        })
                    }),
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && ((!keyboard_event.meta_key() && keyboard_event.ctrl_key()) || (keyboard_event.meta_key() && !keyboard_event.ctrl_key())) && keyboard_event.key().as_str() == "ArrowUp" => {
                            keyboard_event.prevent_default();
                            Msg::MoveNodeUp
                        })
                    }),
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && ((!keyboard_event.meta_key() && keyboard_event.ctrl_key()) || (keyboard_event.meta_key() && !keyboard_event.ctrl_key())) && keyboard_event.key().as_str() == "ArrowDown" => {
                            keyboard_event.prevent_default();
                            Msg::MoveNodeDown
                        })
                    }),
                ],
            ],
            div![
                C!["node-children"],
                s().margin_left(px(10))
                    .border_left(CssBorderLeft::Border(CssBorderWidth::Length(px(1)), CssBorderStyle::Solid, CssColor::Rgba(0., 0., 0., 0.4)))
                    .padding_left(px(20)),
                IF!(!node.folded && vertex.children(tree).next().is_some() => view_nodes(tree, &vertex, editing_node)),
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
