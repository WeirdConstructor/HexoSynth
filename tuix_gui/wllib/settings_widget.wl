!@wlambda;
!@import std;
!@import vizia;
!@import observable;

!popup        = $n;
!popup_entity = $n;

!init_global_settings_popup = {
    .popup        = vizia:new_popup     ${ class = :setting_popup };
    .popup_entity = vizia:new_col popup ${ class = :setting_popup };
};

!@export init_global_settings_popup = init_global_settings_popup;

# * settings - $[index => label, ...]
# * cb       - Callback
!global_settings_popup = {!(settings, cb) = @;
    vizia:remove_all_childs popup_entity;

    iter setting settings {
        !idx = setting.0;
        vizia:new_button
            popup_entity setting.1 {
                cb idx;
                vizia:emit_to 0 popup $p(:popup:close, $n);
                vizia:remove_all_childs popup_entity;
            }
            ${ class = :setting_popup_btn };
    };

    vizia:emit_to 0 popup $p(:popup:open_at_cursor, $n);
};

!@export SettingsWidget = ${
    # * settings - $[index => label, ...]
    _proto = observable:Observable,
    new = {!(settings) = @;
        !self = $&& ${
            _proto = $self,
            _data = $&& ${
                settings = settings,

                root     = $n,
                pop_btn  = $n,
                cur_idx  = 0,
            },
        };

        self.observable_init[];

        self;
    },
    update = {!(idx) = @; $data.cur_idx = idx; },
    update_popbtn = {
        vizia:set_text
            $data.pop_btn
            $data.settings.($data.cur_idx).1;
    },
    build = {!(parent) = @;
        !self = $w& $self;
        !data = $w& $data;

        !root_widget = vizia:new_row parent;
        $data.root = root_widget;
        $self.dropper = std:to_drop { vizia:remove root_widget };

        !popbtn_prev = vizia:new_button $data.root "<" {
            data.cur_idx -= 1;
            data.cur_idx =
                if data.cur_idx < 0
                    (len[data.settings] - 1)
                    data.cur_idx;
            self.update_popbtn[];
            self.emit :changed data.cur_idx;
        } ${ class = :popup_setting_btn_prev };

        $data.pop_btn = vizia:new_button $data.root "popup param" {||
            global_settings_popup data.settings {!(idx) = @;
                std:displayln "Choosen:" idx;
                data.cur_idx = idx;
                self.update_popbtn[];
                self.emit :changed data.cur_idx;
            };
        } ${ class = :popup_setting_btn };

        !popbtn_next = vizia:new_button $data.root ">" {
            data.cur_idx += 1;
            data.cur_idx =
                if data.cur_idx >= len[data.settings]
                    0
                    data.cur_idx;
            self.update_popbtn[];
            self.emit :changed data.cur_idx;
        } ${ class = :popup_setting_btn_next  };

        $self.update_popbtn[];

        $data.root
    },
};