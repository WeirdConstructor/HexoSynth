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
    !pos = _1.pos + $f(1.0, 1.0);
    _.mouse_press_at pos :left;
    _.mouse_release_at pos :left;
};

!do_hover= {
    !pos = _1.pos + $f(1.0, 1.0);
    _.mouse_to pos;
};

!click_on_source_label = {!(td, source, label) = @;
    with_first
        td.list_labels[]
        { _.source == source &and _.label == label }
        { do_click _ td; }
};

!dump_labels = {!(td) = @;
    iter l td.list_labels[] {
        std:displayln l;
    };
};

!@export list = $[
    $[
        {!(td, labels) = @;
            !res = $S(*:{source=cell_name, label=Amp}) labels;
            do_click td res.0;
        },
        {!(td, labels) = @;
            do_hover td
                ($S(*:{tag=knob, source=value, label=*060*}) labels)
                .0;
        },
        {!(td, labels) = @;
            !doc = ($S(*:{label=*Amp\ gain*}) labels).0;
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
