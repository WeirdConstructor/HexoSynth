#!/usr/bin/env wlambda

!check_file = {!(path) = @;
    !toml = std:deser:toml ~ std:io:file:read_text path;
    if is_some[toml.patch] {
        panic ("Patched file: " path);
    };
};

check_file "Cargo.toml";
check_file "hexosynth/Cargo.toml";
check_file "hexosynth_plug/Cargo.toml";
check_file "hexosynth_jack/Cargo.toml";
check_file "hexosynth_cpal/Cargo.toml";
