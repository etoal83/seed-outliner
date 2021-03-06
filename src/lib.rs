#![allow(clippy::wildcard_imports)]
// TODO: Remove
#![allow(dead_code, unused_variables)]

use generational_indextree::{Arena, NodeId as Vertex};
use seed::{prelude::*, *};
use seed_styles::{*, pc, px, rem};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

const TREE_STORAGE_KEY: &str = "seed-outliner-tree";

// ------ ------
//     Init
// ------ ------

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    init_global_styles();

    let mut tree: Arena<Node> = LocalStorage::get(TREE_STORAGE_KEY).unwrap_or(Arena::new());

    let root = if tree.is_empty() {
        let root = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "🌱 Seed Outliner".to_string(),
            folded: false,
        });

        // Initial contents for getting started
        let node_welcome = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "Welcome to Seed-Outliner!".to_owned(),
            folded: false,
        });
        let node_guide_start_edit = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "You can click [[ here ]] to start editing contents.".to_owned(),
            folded: false,
        });
        let node_guide_indent = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "You can indent items by pressing \"Tab\" key.".to_owned(),
            folded: false,
        });
        let node_guide_unindent = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "...and unindent by pressing \"Shift + Tab\" key.".to_owned(),
            folded: false,
        });
        let node_guide_swap = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "You can move items up/down by pressing \"Ctrl + ↑↓\" keys.".to_owned(),
            folded: false,
        });
        let node_guide_lets_move_down = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "Let's try to move this item down to the next item!".to_owned(),
            folded: false,
        });
        let node_guide_move_down_dest = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "Items can be moved in the same level.".to_owned(),
            folded: false,
        });
        let node_guide_fold = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "← You can click this bullet to fold/unfold children items".to_owned(),
            folded: false,
        });
        let node_guide_fold_child1 = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "It's important to hide contents you're not focusing on".to_owned(),
            folded: false,
        });
        let node_guide_fold_child2 = tree.new_node(Node {
            id: Uuid::new_v4(),
            content: "...as well as to show every content you have.".to_owned(),
            folded: false,
        });

        root.append(node_welcome, &mut tree);
        root.append(node_guide_start_edit, &mut tree);
        root.append(node_guide_indent, &mut tree);
        node_guide_indent.append(node_guide_unindent, &mut tree);
        root.append(node_guide_swap, &mut tree);
        node_guide_swap.append(node_guide_lets_move_down, &mut tree);
        root.append(node_guide_fold, &mut tree);
        node_guide_fold.append(node_guide_fold_child1, &mut tree);
        node_guide_fold.append(node_guide_fold_child2, &mut tree);

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

pub fn init_global_styles() {
    GlobalStyle::default()
        .style(
            "a,ul,li,div,p,h1,h2,h3,h4,li,dd,dt,button,label,input",
            s().font_family("-apple-system, 'BlinkMacSystemFont', 'Hiragino Kaku Gothic ProN', 'Hiragino Sans', Meiryo, sans-serif, 'Segoe UI Emoji'")
                .webkit_font_smoothing_antialiased(),
        )
        .style("img", s().box_sizing_content_box())
        .style("*, *:before, *:after", s().box_sizing("inherit"))
        .activate_init_styles();
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

#[derive(Debug, PartialEq)]
enum ArrowKey {
    Left,
    Right,
    Up,
    Down,
}

impl FromStr for ArrowKey {
    type Err = ();

    fn from_str(input: &str) -> Result<ArrowKey, Self::Err> {
        match input {
            "ArrowLeft"  => Ok(ArrowKey::Left),
            "ArrowRight" => Ok(ArrowKey::Right),
            "ArrowUp"    => Ok(ArrowKey::Up),
            "ArrowDown"  => Ok(ArrowKey::Down),
            _ => Err(()),
        }
    }
}


// ------ ------
//    Update
// ------ ------

enum Msg {
    StartEditingNodeContent(Option<Vertex>),
    SetCaretPositionAt(u32),
    EditingNodeContentChanged(String),
    PasteTextIntoEditingNode(String),
    SaveEditedNodeContent,
    InsertNewNode,
    DeleteNodeBackward,
    RemoveNode(Vertex),
    MoveCaretToPreviousNode(ArrowKey),
    MoveCaretToNextNode(ArrowKey),
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
                    let content_element = content_element.get().expect_throw("content_element");

                    content_element
                        .focus()
                        .expect("focus content_element");
                });
            }
        },
        Msg::SetCaretPositionAt(i) => {
            if let Some(editing_node) = &mut model.editing_node {
                let content_element = editing_node.content_element.clone();

                orders.after_next_render(move |_| {
                    let content_element = content_element.get().expect_throw("acquire content element");
                    let selection = document().get_selection().expect_throw("get selection").unwrap();
                    let range = document().create_range().expect_throw("create range");
                    range
                        .set_start(&content_element.child_nodes().get(0).unwrap(), i)
                        .expect_throw("Range: set start");
                    range.collapse();    
                    selection
                        .remove_all_ranges()
                        .expect_throw("Selection: remove all ranges");
                    selection
                        .add_range(&range)
                        .expect_throw("Selection: add range")
                });
            }
        }
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
        Msg::PasteTextIntoEditingNode(text) => {
            log!("PasteTextIntoNode", text);
            if let Some(mut editing_node) = model.editing_node.take() {
                let node = model.tree.get_mut(editing_node.vertex).unwrap().get_mut();
                let caret_position = editing_node.caret_position;
                let (i, _) = editing_node.content
                    .char_indices()
                    .nth(caret_position as usize)
                    .unwrap_or((editing_node.content.chars().count(), ' '));
                editing_node.content.replace_range(i..i, &text);
                node.content = editing_node.content.to_owned();
                let caret_position_dest = caret_position + text.chars().count() as u32;

                let content_element = ElRef::new();

                model.editing_node = Some(EditingNode {
                    id: node.id,
                    content: node.content.clone(),
                    content_element: content_element.clone(),
                    vertex: editing_node.vertex,
                    caret_position: caret_position_dest,
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
                let s = editing_node.content.chars().collect::<Vec<_>>();
                let (left, right) = s.split_at(caret_position as usize);
                let left_content = left.iter().collect::<String>().to_owned();
                let right_content = right.iter().collect::<String>().to_owned();
                orders.send_msg(Msg::EditingNodeContentChanged(left_content));
                orders.send_msg(Msg::SaveEditedNodeContent);

                let new_id = Uuid::new_v4();
                
                let new_node = model.tree.new_node(Node {
                    id: new_id,
                    content: right_content,
                    folded: false,
                });

                match editing_node.vertex.children(&model.tree).next() {
                    Some(child) => editing_node.vertex.prepend(new_node, &mut model.tree),
                    None => editing_node.vertex.insert_after(new_node, &mut model.tree),
                };
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
                    let caret_position_dest = node.content.chars().count() as u32;
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
        Msg::MoveCaretToPreviousNode(key) => {
            log!("MoveCaretToPreviousNode");
            if let Some(editing_node) = &mut model.editing_node {
                let previous_sibling = editing_node.vertex.preceding_siblings(&model.tree).skip(1).next();
                let parent = editing_node.vertex.ancestors(&model.tree).skip(1).next();
                let destination = match (previous_sibling, parent) {
                    (Some(sibling), _) => sibling.descendants(&model.tree).last(),
                    (None, Some(parent)) => if parent != model.root { Some(parent) } else { None },
                    (_, _) => None,
                };
                
                if let Some(vertex) = destination {
                    orders.send_msg(Msg::SaveEditedNodeContent);
                    orders.send_msg(Msg::StartEditingNodeContent(Some(vertex)));
                    orders.force_render_now();

                    let node = model.tree.get(vertex).unwrap().get();

                    match key {
                        ArrowKey::Left => {
                            orders.send_msg(Msg::SetCaretPositionAt(node.content.chars().count() as u32));
                        },
                        ArrowKey::Up => {
                            orders.send_msg(Msg::SetCaretPositionAt(0));
                        },
                        _ => (),
                    };
                }
            }
        },
        Msg::MoveCaretToNextNode(key) => {
            log!("MoveCaretToNextNode");
            if let Some(editing_node) = &mut model.editing_node {
                let children = editing_node.vertex.children(&model.tree).next();
                let next_sibling = editing_node.vertex.following_siblings(&model.tree).skip(1).next();
                let ancestors_next_sibling = editing_node.vertex.ancestors(&model.tree).find_map(|vtx| model.tree[vtx].next_sibling());
                let destination = match (children, next_sibling, ancestors_next_sibling) {
                    (Some(child), _, _) => Some(child),
                    (None, Some(sibling), _) => Some(sibling),
                    (None, None, Some(neighbor)) => Some(neighbor),
                    (_, _, _) => None,
                };
                
                if let Some(vertex) = destination {
                    orders.send_msg(Msg::SaveEditedNodeContent);
                    orders.send_msg(Msg::StartEditingNodeContent(Some(vertex)));
                    orders.force_render_now();

                    let node = model.tree.get(vertex).unwrap().get();

                    match key {
                        ArrowKey::Right => {
                            orders.send_msg(Msg::SetCaretPositionAt(0));
                        },
                        ArrowKey::Down => {
                            orders.send_msg(Msg::SetCaretPositionAt(node.content.chars().count() as u32));
                        },
                        _ => (),
                    };
                }
            }
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

fn view(model: &Model) -> seed::virtual_dom::Node<Msg> {
    div![
        s().padding_x(px(6)),
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
        let caret_at_head = editing_node.map_or(false, |editing_node| editing_node.caret_position == 0);
        let caret_at_tail = editing_node.map_or(false, |editing_node| editing_node.caret_position == editing_node.content.chars().count() as u32);

        div![
            C!["node"],
            div![
                C!["node-self"],
                el_key(&node.id),
                s().padding_y(rem(0.3))
                    .padding_x(px(5)),
                a![
                    C!["node-bullet"],
                    s().display(CssDisplay::InlineBlock)
                        .text_align(CssTextAlign::Center)
                        .vertical_align(CssVerticalAlign::Top)
                        .width(px(18))
                        .border_radius(CssBorderRadius::Length(px(2))),
                    IF!(vertex.children(tree).next().is_some() => s().hover().bg_color(CssColor::Rgba(189.0, 195.0, 199.0, 1.0))),
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
                    ev(Ev::Paste, |event| {
                        event.prevent_default();
                        let clipboard_event = event.dyn_ref::<web_sys::ClipboardEvent>().unwrap_throw();
                        let text = clipboard_event.clipboard_data().unwrap().get_data("text/plain").unwrap();
                        Msg::PasteTextIntoEditingNode(text)
                    }),
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && keyboard_event.key_code() != 229 && keyboard_event.key().as_str() == "Enter" => {
                            keyboard_event.prevent_default();
                            Msg::InsertNewNode
                        })
                    }),
                    keyboard_ev(Ev::KeyDown, move |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && keyboard_event.key_code() != 229 && caret_at_head && keyboard_event.key().as_str() == "Backspace" => {
                            Msg::DeleteNodeBackward
                        })
                    }),
                    keyboard_ev(Ev::KeyDown, move |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && caret_at_head && (!keyboard_event.meta_key() || !keyboard_event.ctrl_key()) && (keyboard_event.key().as_str() == "ArrowUp" || keyboard_event.key().as_str() == "ArrowLeft") => {
                            let key = ArrowKey::from_str(keyboard_event.key().as_str()).unwrap();
                            Msg::MoveCaretToPreviousNode(key)
                        })
                    }),
                    keyboard_ev(Ev::KeyDown, move |keyboard_event| {
                        IF!(!keyboard_event.is_composing() && caret_at_tail && (!keyboard_event.meta_key() || !keyboard_event.ctrl_key()) && (keyboard_event.key().as_str() == "ArrowDown" || keyboard_event.key().as_str() == "ArrowRight") => {
                            let key = ArrowKey::from_str(keyboard_event.key().as_str()).unwrap();
                            Msg::MoveCaretToNextNode(key)
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
                s().margin_left(px(13))
                    .border_left(CssBorderLeft::Border(CssBorderWidth::Length(px(1)), CssBorderStyle::Solid, CssColor::Rgba(0., 0., 0., 0.4)))
                    .padding_left(px(16)),
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
