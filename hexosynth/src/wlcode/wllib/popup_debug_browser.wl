!@wlambda;
!@import std;
!@import ui;
!@import styling wllib:styling;

styling:add_style :debug_browser ${
    parent = :main_help_wichtext,
    typ    = :rect,
};
styling:add_layout :debug_browser ${
    parent = :main_help_wichtext,
    width = :pixels => 1000,
};

styling:add_style :debug_wichtext ${
    parent = :wichtext,
};
styling:add_layout :debug_wichtext ${
    parent = :wichtext,
    height = :stretch => 1,
};

!@export DebugPanel = ${
    new = {
        ${
            _proto = $self,
            _data = ${
                wichtext_data = ui:wichtext_simple_data_store[],
                value_field = ui:txt_field[],
            },
        }
    },
    build = {
        !panel = styling:new_widget :debug_browser;
        panel.auto_hide[];

        !wt = styling:new_widget :debug_wichtext;
        wt.set_ctrl :wichtext $data.wichtext_data;
        panel.add wt;

        !value_entry = styling:new_widget :value_entry;
        value_entry.set_ctrl :entry $data.value_field;
        panel.add value_entry;

        !self = $self;
        value_entry.reg :enter {!(wid, ev) = @;
            self.update_filter ev;
        };

        $data.panel = panel;

        panel
    },
    update_filter = {|0<1| !(filt_str) = @;
        $data.filt_str = filt_str;

        !text = "```\n\n";

        iter lbl $data.labels {
            !lbl_ser = std:ser:json lbl;

            if is_some[$data.filt_str] &and is_none[lbl_ser $p(0, $data.filt_str)] {
                next[];
            };

            .text +>= lbl_ser +> "\n";
        };

        .text +>= "```\n";
        $data.wichtext_data.set_text ~ ui:mkd2wt text;
    },
    show = {!(labels) = @;
        $data.labels = labels;
        $data.filt_str = $none;
        $data.value_field.set "";
        $self.update_filter[];
        $data.panel.show[];
    },
};
