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

## `$<UI::Widget>` API

### `widget.set_ctrl :graph_minmax $[sample_count, $<UI::GraphMinMaxModel>]`

Creates a new Min/Max Graph widget that is connected to the given `$<UI::GraphMinMaxModel>`.
You can obtain a `$<UI::GraphMinMaxModel>` from the `matrix.create_graph_minmax_monitor`
method. The *sample_count* should be what corresponds to the given `$<UI::GraphMinMaxModel>`,
for instance `hx:MONITOR_MINMAX_SAMPLES`.

## Hexo Synth Module - `hx:`

    !@import hx;

### `hx:MONITOR_MINMAX_SAMPLES` : integer

Returns the number of samples a channel of the monitored cell takes.
This is what you should pass to the `:graph_minmax` widget as samples.

## `$<HexoDSP::Matrix>` API

### `matrix.create_pattern_data_model[tracker_id]` -> `$<UI::PatModel>`

Returns a `$<UI::PatModel>` that can be used in the constructor
of a `:pattern_editor` widget. The pattern data model will be connected
to the corresponding pattern data. That means for instance the node *TSeq 0*
will be connected to the pattern data with ID `0`.

### `matrix.pop_error[]` -> (`$none` or string)

Returns an error message if some error occured recently. You should check this
at best every frame and show the user the error message. That could be for
instance that the backend failed to load a WAV sample.

### `matrix.monitor_cell[cell]`

Sets the monitored cell to `cell`. If you just inserted the cell, make sure
to call `matrix.sync[]` before monitoring it, or else the monitored Node has
not been instanciated yet.

A monitored cell can be displayed on a `:graph_minmax` widget, using the
`$<UI::GraphMinMaxModel>` that can be obtained from
`matrix.create_graph_minmax_monitor`.

### `matrix.monitored_cell[]` -> `cell`

Returns the currently monitored cell.

### `matrix.create_graph_minmax_monitor[index]`

Creates a `$<UI::GraphMinMaxModel>` bound to the monitor at _index_. Where
_index_ must be between 0 and 5. There are currently 6 monitors, the first 3
are for the input signals of the `matrix.monitored_cell[]` and the latter 3
are for the outputs.

### `matrix.get_connections[$i(x, y)]`

Returns a set of connections for the current cell. If there is no connection
it returns `$none`. Otherwise a vector of following elements:

    ${
        center = ${ dir = <celldir to other>, port = <portname> },
        other = ${ dir = <celldir to center>, port = <portname>, pos = $i(other_x, other_y) },
    }
