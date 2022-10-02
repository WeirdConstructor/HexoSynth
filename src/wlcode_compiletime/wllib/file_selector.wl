!@wlambda;
!@import std;
!@import ui;
!@import hx;
!@import styling wllib:styling;

styling:add_style :file_list ${
    bg_color = ui:UI_ACCENT_BG2_CLR,
    color = ui:UI_PRIM_CLR,
    pad_item = 4,
    border2 = 1,
    border_color = ui:UI_ACCENT_CLR,
    color2 = ui:UI_BG2_CLR,
};
styling:add_layout :file_list ${
    height = :pixels => 400,
    width = :pixels => 300,
};

styling:add_style :dir_list ${
    parent = :file_list,
};
styling:add_layout :dir_list ${
    height = :stretch => 1,
    width = :pixels => 300,
};

!@export FileSelector = ${
    new = {!(mode) = @;
        !root_dirs =
            match mode
                :patches => { $[] +> hx:get_directory_patches[]; }
                :samples => { hx:get_directories_samples[]; };

        ${
            _proto = $self,
            _data = ${
                mode = mode,
                file_type = :sample,
                root_dirs = root_dirs,
                cur_path = root_dirs.0,
                path_stack = $[],
                directories = $[],
                files = $[],
                file_list_data = ui:list_data[],
                dir_list_data = ui:list_data[],
                cur_path_data = ui:txt root_dirs.0.1,
            },
        }
    },
    build_selector = {
        !grp = styling:new_widget :file_dialog;

        !file_div = styling:new_widget :file_dialog_panel;

        !dir_list = styling:new_widget :dir_list;
        dir_list.set_ctrl :list $data.dir_list_data;

        !file_list = styling:new_widget :file_list;
        file_list.set_ctrl :list_selector $data.file_list_data;

        !dir_name_lbl = styling:new_widget :dir_name_lbl;
        dir_name_lbl.set_ctrl :label $data.cur_path_data;
        file_div.add dir_name_lbl;
        file_div.add file_list;

        grp.add dir_list;
        grp.add file_div;

        !self = $self;
        dir_list.reg :select {!(wid, idx) = @;
            if idx == 0 {
                self.navigate_parent[];
                return $n;
            };

            !list_idx = idx - 1;
            self.navigate_dir_index list_idx;
        };

        !data = $data;
        file_list.reg :select {!(wid, idx) = @;
            std:displayln "SELECT FILE:" idx data.files.(idx - 1);
        };

        grp
    },
    navigate_parent = {
        if len[$data.path_stack] > 0 {
            $data.cur_path = std:pop $data.path_stack;
            $data.cur_path_data.set $data.cur_path.1;
            $self.update[];
        };
    },
    navigate_dir_index = {!(index) = @;
        !dir = $data.directories.(index);

        std:push $data.path_stack $data.cur_path;
        $data.cur_path = dir;
        $data.cur_path_data.set $data.cur_path.1;
        $self.update[];
    },
    update = {
        !dirs = $[];
        !files = $[];
        !_ = std:fs:read_dir $data.cur_path.0 {!(ent) = @;
            match ent.type
                :f => {
                    match $data.mode
                        :patches => {
                            if ent.name &> $r/*.hxy/ {
                                std:push files $p(ent.path, ent.name);
                            };
                        }
                        :samples => {
                            if ent.name &> $r/*.wav$$/ {
                                std:push files $p(ent.path, ent.name);
                            };
                        };
                }
                :d => {
                    std:push dirs $p(ent.path, ent.name);
                };
            $f
        };

        std:sort { std:cmp:str:asc std:str:to_lowercase[_.1] std:str:to_lowercase[_1.1] } dirs;
        std:sort { std:cmp:str:asc std:str:to_lowercase[_.1] std:str:to_lowercase[_1.1] } files;

        $data.directories = dirs;
        $data.files = files;

        $data.file_list_data.clear[];
        iter f files \$data.file_list_data.push f.1;

        $data.dir_list_data.clear[];
        $data.dir_list_data.push ".. (parent)";
        iter d dirs \$data.dir_list_data.push ~ d.1 "/";
    },
};
