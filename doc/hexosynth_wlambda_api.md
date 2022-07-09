# HexoSynth - WLambda API Reference Manual

This is the reference documentation for the internal WLambda API
of HexoSynth. The goal of this document is to serve as overview
of the available functionality that HexoSynth exposes to WLambda.

It's very likely incomplete. Please refer to the WLambda and Rust source
when in doubt. Or ask Weird Constructor to extend the documentation
of certain parts.

## User Interface Module - `ui:`

    !@import ui;

### `ui:create_pattern_feedback_dummy[]` -> `$<UI::PatEditFb>`

Creates a dummy pattern feedback object. You can pass this to the
constructor of a new `:pattern_editor` widget.

### `ui:create_pattern_data_unconnected[max_rows]` -> `$<UI::PatModel>`

Creates an empty unconnected (not connected to the Matrix backend)
pattern data object with `max_rows` storage capacity.  The pattern
editor might not support entering longer patterns than 256 rows,
mostly due to keyboard commands, which only allow entering 256 as
length.

The return value can be passed to the constructor of a `:pattern_editor` widget.


## Hexo Synth Module - `hx:`

    !@import hx;


## `$<HexoDSP::Matrix>` API

### `matrix.create_pattern_data_model[tracker_id]` -> `$<UI::PatModel>`

Returns a `$<UI::PatModel>` that can be used in the constructor
of a `:pattern_editor` widget. The pattern data model will be connected
to the corresponding pattern data. That means for instance the node *TSeq 0*
will be connected to the pattern data with ID `0`.
