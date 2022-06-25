!@wlambda;
!@import std;

!inside_rect = {!(target, test) = @;
         test.x >= target.x
    &and test.y >= target.y
    &and test.x <= (target.x + target.2)
    &and test.y <= (target.y + target.3)
};

!with_first = {!(list, filt, do) = @;
    iter l list {
        if (filt l) {
            return ~ $o(do l);
        };
    };
    $o()
};

!do_click = {
    !pos = _.pos + $f(1.0, 1.0);
    _1.mouse_press_at pos :left;
    _1.mouse_release_at pos :left;
};

!do_hover= {
    !pos = _.pos + $f(1.0, 1.0);
    _1.mouse_to pos;
};

!click_on_source_label = {!(td, source, label) = @;
    with_first
        td.list_labels[]
        { _.source == source &and _.label == label }
        { do_click _ td; }
};

!@export list = $[
    $[
        {!(td) = @;
            click_on_source_label td "cell_name" "Amp";
        },
        {!(td) = @;
            with_first
                _.list_labels[]
                { is_some ~ (std:str:find "060" _.label) }
                { do_hover _ td; }
        },
        {!(td) = @;
            !doc =
                with_first _.list_labels[]
                    { is_some ~ std:str:find "Amp gain" _.label }
                    { _ };
            std:assert doc "FOO";
        },
    ],
    $[
        { std:displayln "XXX STEP 1" },
        { std:displayln "XXX STEP 2";
#            iter lbl _.list_labels[] {
#                std:displayln lbl;
#            };
        },
    ],
];
