# This script generates simple Rust code for setting up
# more complicated test graphs for HexoDSP test cases.
#
# The output has to be adjusted, to list proper NodeIds and
# input/output port name instead of the indices.

if len[@@.0] == 0 {
    std:displayln "usage: wlambda patch2testcode.wl init.hxy";
    return;
};

!patch = std:deser:json ~ std:io:file:read_text @@.0;

std:displayln ~ std:ser:json patch;

iter cell patch.cells {
    !(typ, inst, x, y, ins, outs) = cell;

    !varname = std:str:cat typ "_" inst + 1;
    std:displayln ~
        $F "let {:7} = NodeId::{}({});"
            varname typ inst;
};

!print_ports = {!(varname, celldir, vardir, ports) = @;
    !s = "";

    !one_in = $false;
    iter port ports {
        !first = not[one_in];
        if first {
            .s      = s "\n    ." celldir "(";
            .one_in = $t;
        };

        if not[first] {
            .s = s ", ";
        };

        if port >= 0 {
            .s = s varname "." vardir "(\"" str[port] "\")";
        } {
            .s = s "None";
        };
    };

    s ")"
};

iter cell patch.cells {
    !(typ, inst, x, y, ins, outs) = cell;

    !varname = std:str:cat typ "_" inst + 1;

    !s = $F "matrix.place({}, {},\n    Cell::empty({})"
            x y varname;

    .s = s ~ print_ports varname "input" "inp" ins;
    .s = s ~ print_ports varname "out" "out" outs;

    .s = s ");";
    std:displayln s;
};
