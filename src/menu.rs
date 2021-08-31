// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::state::{ItemType, MenuItem, RandSpecifier, UICategory};
use hexodsp::{NodeInfo, NodeId, Cell};

#[derive(Debug, Clone)]
pub enum MenuState {
    None,
    SelectCategory { user_state: i64 },
    SelectNodeIdFromCat { category: UICategory, user_state: i64 },
    SelectOutputParam { node_id: NodeId, node_info: NodeInfo, user_state: i64 },
    SelectInputParam { node_id: NodeId, node_info: NodeInfo, user_state: i64 },
    ContextActionPos {
        pos: (usize, usize),
    },
    ContextAction {
        cell: Cell,
        node_id: NodeId,
        node_info: NodeInfo
    },
    ContextRandomSubMenu {
        cell: Cell,
    },
    ContextRandomPosSubMenu {
        pos: (usize, usize),
    },
}

impl MenuState {
    pub fn to_items(&self) -> Vec<MenuItem> {
        match self {
            MenuState::None => vec![],
            MenuState::SelectCategory { user_state } => {
                vec![
                    if *user_state > 0 {
                        MenuItem {
                            typ:    ItemType::Back,
                            label:  "<Back".to_string(),
                            help:   "Back\nBack to previous menu".to_string(),
                        }
                    } else {
                        MenuItem {
                            typ:    ItemType::Back,
                            label:  "<Exit".to_string(),
                            help:   "\nExit Menu".to_string(),
                        }
                    },
                    MenuItem {
                        typ:    ItemType::Category(UICategory::Osc),
                        label:  "Osc".to_string(),
                        help:   "Osc\nAudio oscillators.".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::Signal),
                        label: "Signal".to_string(),
                        help:  "Signal\nSignal shapers:\n- Filters\n- Waveshapers\n- Delays".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::Ctrl),
                        label: "Ctrl".to_string(),
                        help: "Ctrl\nControl signal shapers:\n- Ctrl converters\n- Quantizers\n- Sample & Hold\n- Slew".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::Mod),
                        label: "Mod".to_string(),
                        help: "Mod\nModulation sources:\n- Envelopes\n- LFOs\n- Sequencers\n- Utils".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::NtoM),
                        label: "N->M".to_string(),
                        help: "N->M\nSignal merge and splitters:\n- Mixers\n- Logic\n- Math\n- Multiplexers".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::IOUtil),
                        label: "I/O".to_string(),
                        help: "I/O\nExternal connections:\n- Audio I/O\n- MIDI".to_string(),
                    },
                ]

//                for i in 7..44 {
//                    v.push(
//                    MenuItem {
//                        typ: ItemType::Category(UICategory::IOUtil),
//                        label: format!("X {}", i),
//                        help: "I/O\nExternal connections:\n- Audio I/O\n- MIDI".to_string(),
//                    });
//                }
            },
            MenuState::SelectNodeIdFromCat { category, .. } => {
                let mut items = vec![];
                items.push(MenuItem {
                    typ:    ItemType::Back,
                    label:  "<Back".to_string(),
                    help:   "\nBack to previous menu".to_string(),
                });

                category.get_node_ids(0, |node_id| {
                    items.push(
                        MenuItem {
                            typ:    ItemType::NodeId(node_id),
                            label:  node_id.label().to_string(),
                            help:   NodeInfo::from_node_id(node_id).desc().to_string(),
                        },
                    );
                });
                items
            },
            MenuState::SelectOutputParam { node_id: _, node_info, .. } => {
                let mut items = vec![];
                items.push(MenuItem {
                    typ:    ItemType::Back,
                    label:  "<Back".to_string(),
                    help:   "\nBack to previous menu".to_string(),
                });

                for out_idx in 0..node_info.out_count() {
                    items.push(
                        MenuItem {
                            typ:    ItemType::OutputIdx(out_idx),
                            label:  node_info.out_name(out_idx).unwrap_or("?").to_string(),
                            help:   node_info.out_help(out_idx).unwrap_or("?").to_string(),
                        },
                    );
                }

                items
            },
            MenuState::SelectInputParam { node_id: _, node_info, .. } => {
                let mut items = vec![];
                items.push(MenuItem {
                    typ:    ItemType::Back,
                    label:  "<Back".to_string(),
                    help:   "\nBack to previous menu".to_string(),
                });

                for out_idx in 0..node_info.in_count() {
                    items.push(
                        MenuItem {
                            typ:    ItemType::InputIdx(out_idx),
                            label:  node_info.in_name(out_idx).unwrap_or("?").to_string(),
                            help:   node_info.in_help(out_idx).unwrap_or("?").to_string(),
                        },
                    );
                }

                items
            },
            MenuState::ContextActionPos { pos } => {
                vec![
                    MenuItem {
                        typ:    ItemType::Back,
                        label:  "<Back".to_string(),
                        help:   "\nBack to previous menu".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::SubMenu {
                            menu_state:
                                Box::new(
                                    MenuState::ContextRandomPosSubMenu {
                                        pos: *pos
                                    }),
                            title: "Randomized Stuff".to_string(),
                        },
                        label:  "Rand>".to_string(),
                        help:   "Rand>\nAccess to all kinds of random actions with the current position.".to_string(),
                    },
                ]
            },
            MenuState::ContextAction { cell, .. } => {
                vec![
                    MenuItem {
                        typ:    ItemType::Back,
                        label:  "<Back".to_string(),
                        help:   "\nBack to previous menu".to_string(),
                    },
                    MenuItem {
                        typ:    ItemType::Help(cell.node_id()),
                        label:  "Help".to_string(),
                        help:   "Node Help\nJumps directly to the 'Node' \
                                 tab in the help screen for this node.".to_string(),
                    },
                    MenuItem {
                        typ:    ItemType::Delete,
                        label:  "Delete".to_string(),
                        help:   "Delete\nDeletes/clears the matrix cell.".to_string(),
                    },
                    MenuItem {
                        typ:    ItemType::ClearPorts,
                        label:  "Clear Ports".to_string(),
                        help:   "Clear Ports\nClear unused inputs & outputs of this cell.".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::SubMenu {
                            menu_state:
                                Box::new(
                                    MenuState::ContextRandomSubMenu {
                                        cell: *cell
                                    }),
                            title: "Randomized Stuff".to_string(),
                        },
                        label:  "Rand>".to_string(),
                        help:   "Rand>\nAccess to all kinds of random actions with the current cell.".to_string(),
                    },
                ]
            },
            MenuState::ContextRandomSubMenu { cell: _ } => {
                vec![
                    MenuItem {
                        typ:    ItemType::Back,
                        label:  "<Back".to_string(),
                        help:   "\nBack to previous menu".to_string(),
                    },
                    MenuItem {
                        typ:    ItemType::RandomNode(RandSpecifier::One),
                        label:  "Rand Node".to_string(),
                        help:   "Rand Node\nSpawn a random node adjacent to \
                                 the current cell. Use this either as a \
                                 challenge to make use of the spawned node or \
                                 just as an inspiration.".to_string(),
                    },
                ]
            },
            MenuState::ContextRandomPosSubMenu { pos: _ } => {
                vec![
                    MenuItem {
                        typ:    ItemType::Back,
                        label:  "<Back".to_string(),
                        help:   "\nBack to previous menu".to_string(),
                    },
                    MenuItem {
                        typ:    ItemType::RandomNode(RandSpecifier::One),
                        label:  "Rand Node".to_string(),
                        help:   "Rand Node\nSpawn a random node adjacent to \
                                 the current cell. Use this either as a \
                                 challenge to make use of the spawned node or \
                                 just as an inspiration.".to_string(),
                    },
                    MenuItem {
                        typ:    ItemType::RandomNode(RandSpecifier::Six),
                        label:  "Rand 6".to_string(),
                        help:   "Rand 6\nSpawn 6 random nodes around this position.\
                                 Use this either as a \
                                 challenge to make use of the spawned nodes or \
                                 just as an inspiration.".to_string(),
                    },
                ]
            },
        }
    }
}
