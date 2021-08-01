use crate::state::{ItemType, MenuItem, UICategory};
use hexodsp::{NodeInfo, NodeId, Cell};

#[derive(Debug, Clone)]
pub enum MenuState {
    None,
    SelectCategory { user_state: i64 },
    SelectNodeIdFromCat { category: UICategory, user_state: i64 },
    SelectOutputParam { node_id: NodeId, node_info: NodeInfo, user_state: i64 },
    SelectInputParam { node_id: NodeId, node_info: NodeInfo, user_state: i64 },
    ContextAction {
        cell: Cell,
        node_id: NodeId,
        node_info: NodeInfo
    },
}

impl MenuState {
    pub fn to_items(&self) -> Vec<MenuItem> {
        match self {
            MenuState::None => vec![],
            MenuState::SelectCategory { .. } => {
                let mut v =
                vec![
                    MenuItem {
                        typ:    ItemType::Back,
                        label:  "<Exit".to_string(),
                        help:   "\nExit Menu".to_string(),
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
                        typ: ItemType::Category(UICategory::CV),
                        label: "CV".to_string(),
                        help: "CV\nControl voltage shapers:\n- CV converters\n- Quantizers\n- Sample & Hold\n- Slew".to_string(),
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
                ];

//                for i in 7..44 {
//                    v.push(
//                    MenuItem {
//                        typ: ItemType::Category(UICategory::IOUtil),
//                        label: format!("X {}", i),
//                        help: "I/O\nExternal connections:\n- Audio I/O\n- MIDI".to_string(),
//                    });
//                }

                v
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
            MenuState::SelectOutputParam { node_id, node_info, .. } => {
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
            MenuState::SelectInputParam { node_id, node_info, .. } => {
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
            MenuState::ContextAction { cell, .. } => {
                vec![
                    MenuItem {
                        typ:    ItemType::Back,
                        label:  "<Back".to_string(),
                        help:   "\nBack to previous menu".to_string(),
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
                ]
            },
        }
    }
}
