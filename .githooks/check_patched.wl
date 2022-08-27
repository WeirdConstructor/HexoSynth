#!/usr/bin/env wlambda

!check_file = {!(path) = @;
    !toml = std:deser:toml ~ std:io:file:read_text path;
    if is_some[toml.patch] {
        panic ("Patched file: " path);
    };
};

check_file "Cargo.toml";
check_file "nih_plug/Cargo.toml";
check_file "jack_standalone/Cargo.toml";
check_file "cpal_standalone/Cargo.toml";
