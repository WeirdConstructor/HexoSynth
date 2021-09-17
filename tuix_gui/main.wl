

!:global init = {!(ui) = @;
    !par = ui.new_row 0;

    !i = 0;
    !btn = $n;

    .btn = ui.new_button par "Test" {
        .i += 1;

        std:displayln "CLICK:" i;

        _.set_text btn ~ $F "Counter: {}" i;
        _.redraw[];
    };
};
