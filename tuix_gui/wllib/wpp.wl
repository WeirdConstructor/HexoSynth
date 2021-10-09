!@wlambda;
!@import std;

!run_var_expr_lang = {!(vars, text) = @;
    $rs/\$\((^$+[<_A-Z]) $*$s (^$+$S) $*$s (^$*[^\)])\)/ text {!(m, offs, _l) = @;
        #d# std:displayln "APPLY EXPR:" m;

        !expr_body = m.3;
        iter e vars {
            .expr_body = $p(e.1, e.0) expr_body;
        };

        !args = $@v iter a ((" " => 0) expr_body) {
            !s = std:str:trim a;
            if len[s] > 0 \$+ s;
        };

        !key = "$" m.1;

        #d# std:displayln "ARGS:" key expr_body args;

        !res =
            match m.2
                "+" => { float[args.0] + float[args.1] }
                "-" => { float[args.0] - float[args.1] }
                "*" => { float[args.0] * float[args.1] }
                "/" => { float[args.0] / float[args.1] }
                { panic ("Unknown expr fun:" m.2) };

        if key == "$<" {
            res
        } {
            vars.(key) = str res;
            ""
        };
    }
};

!run_macro_lang = {!(functions, text) = @;

    .text = $rs/!(^$*[_A-Z]) $*$s \((^$*[^\)])\) $*$s \{\{ (^$<*?) \}\}/ text {!(m, offs, _l) = @;
        functions.(m.1) = $[
            (',' => 0) m.2,         # args
            (std:str:trim m.3) "\n" # body
        ];
        ""
    };

    .text = $rs/\$(^$*[_A-Z])$*$s\((^$*[^\}])\)/ text {!(m, offs, _l) = @;
        !fun = functions.(m.1);

        if is_none[fun] {
            panic ("UNDEFINED MACRO IN STYLE:" m.1);
        };

        !keys     = fun.0;
        !args     = (',' => 0) m.2;
        !arg_map  = ${};
        !i        = 0;
        !fun_body = fun.1;

        !vars = ${};
        !order = $[];

        iter k keys {
            !key         = "$" ~ std:str:trim k;
            !replacement = std:str:trim args.(i);

            vars.(key) = replacement;
            std:push order key;

            .i += 1;
        };

        .fun_body = run_var_expr_lang vars fun_body;

        iter e vars {
            .fun_body = (e.1 => str[e.0]) fun_body;
        };

        fun_body
    };

    text
};

!@export run_macro_lang = run_macro_lang;
