!@import wpp;

std:displayln ~
    wpp:run_macro_lang
        ${}
        std:io:file:read_text["main_style_test.css"];
""
