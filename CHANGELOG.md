0.2.0-alpha-1 (unreleased)
==========================

Complete rewrite of the GUI was done since 0.1.0-alpha-3. As well as lots of
little features added.

* Feature: New node completed: 'PVerb'
* Feature: Mouse wheel now also moves knobs.
* Feature: New node added: 'Mux9' a 9 channel multiplexer/switch.
* Feature: New node added: 'CQnt' a control signal pitch quantizer.
* Feature: New node added: 'Quant' a pitch signal quantizer.
* Feature: New node added: 'FormFM' a formant FM synthesizer.
* Feature: MIDI note and CC input with a MIDI log window.
* Feature: 'Code' node added and the WBlockDSP visual DSP programming language that
is compiled just-in-time (JIT) to machine code and executed in the audio thread.
* Change: Moved 'TSeq' from the 'CV' to the 'Mod' category.
* Change: RndWk did not properly reflect back the overshoots.
Now it behaves more in tune with the 'step' setting and does not
suddenly jump to the 'min' anymore if exceeding the 'max'.
* Change: Triggers react to 0.5 and not 0.75 now. This is because
I wanted to have the same logic level for triggers as for other logic
operations.
* Change: Presets now store the denormalized values, to have better
compatibility in future if the parameter ranges or mapping changes.
* Change: Renamed CV to Ctrl (aka Control Signal) for more consistency
within HexoSynth.
* Documentation: 'PVerb' node now has a complete documentation.
* Bugfix: Setting the 'PVerb' 'predly' parameter to 0.0 did not work
correctly and acted as very very long delay.
It will now just skip the delay entirely.
* Bugfix: The Dattorro 'PVerb' tend to blow up if you set 'size' to a
very small (<0.1) value!
* Bugfix: The low pass filter for oversampling did a slightly wrong Q
calculation. Was not audible though.
* Bugfix: 'PVerb' now also properly handles if only one input
channel is connected.
* Bugfix: TriSawLFO (TsLFO) node did output too high values if the `rev`
parameter was changed or modulated at runtime.
* Bugfix: Found a bug in cubic interpolation in the sample player and
similar bugs in the delay line (and all-pass & comb filters). Refactored
the cubic interpolation and tested it seperately now.
* Feature: Added Scope DSP node and view in GUI and NodeConfigurator/Matrix
API for retrieving the scope handles for access to it's capture buffers.

0.1.0-alpha-3 (2021-08-13)
==========================

* Feature: New node added: 'Mix3' a simple 3 channel mixer node
to sum 3 signals.
* Feature: New node added: 'BOsc' a (B)asic (Osc)illator
for band-limited sine, triangle, saw and pulse waveforms.
* Feature: New node added: 'VOsc' a (V)ector Phase Shaping oscillator
with overdrive and oversampling.
* Feature: New node added: 'Comb' a Comb Filter.
* Feature: New node added: 'TsLFO' a Triangle/Saw LFO with an adjustable
waveform.
* Feature: TSeq module documentation shows a value cheat sheet,
to quickly compose gates and values in your sequences.
* Feature: The min/max signal monitors also print the min/max/average values
of the signal that is visible in the monitor.
* Feature: **Completely new interaction with the matrix**.
  - Left mouse click creates new cell.
  - Right mouse click opens a context menu.
  - Right mouse drag of filled cell to an empty will move the entire cluster
    of connected cells.
  - Left mouse drag from empty cell to adjacent filled cell lets you create
    a new node with default ports.
  - Right mouse drag from filled cell to empty moves the cell.
  - Right mouse drag from empty cell to adjacent filled cell lets you create
    a new node with explicitly selected ports.
  - Right mouse drag of filled cell to adjacent connected cell will
    split the connected cluster and make room for a new node.
  - Left mouse drag of between two adjacent empty cells lets you instantiate
    two new nodes with default input/outputs.
  - Right mouse drag of between two adjacent empty cells lets you instantiate
    two new nodes with explicitly selected input/outputs.
  - Left mouse drag from empty to non adjacent filled cell creates a linked copy.
  - Right mouse drag from empty to non adjacent filled cell creates a new node instance.
  - Left mouse drag from existing cell to a non adjacent existing cell
    creates a linked copy around the destination cell.
  - Right mouse drag from existing cell to a non adjacent existing cell
    creates a new instance of the source cell node around the destination cell.
* Feature: Context menus come with random node generation functionality.
* Feature: Delete a node/cell is now in the context menu.
* Feature: Clear unused ports of a cell can be found in the context menu too.
* Feature: Added context menu "Help" entry, to quickly jump to the help of
the corresponding node.
* Feature: Added temporary "Save" button to the UI.
* Change: SFilter - removed the other Stilson/Moog variants (High/Band/Notch)
and implemented a different low pass variant, that seems to be slightly more
stable.
* Change: Relicensed the whole project to **GPL-3.0-or-later**.
* Change: The middle mouse button is now responsible for panning the matrix.
* Change: The scroll wheel allows zooming in/out of the matrix.
* Change: The min/max signal monitors are now wider and display 3 seconds
of the signal instead of only 2.
* Change: The patch file format now stores input/output port names
now instead of indices. Current format with port indices is still
loaded correctly.
* Change: 'Sin' node now has a randomized initial phase, except for the
very first instance 'Sin(0)'.
* Change: The signal scopes draw the center line no longer above the
waveform.
* Change: The font size of the node name inside the hex cells is automatically
determined now.
* Bugfix: Note columns in the tracker did not show the note name.
* Bugfix: The all-pass filter of the AllP node had a bad all-pass implementation.
* Bugfix: The delay line interpolation had an off-by-1 bug that lead to
a very distorted sound when modulating the delay line.
* Project: GUI test suite can now place matrix cells directly.
* Project: Moved GUI tests suite to it's own sub directory 'gui\_tests'
as separate application.

0.1.0-alpha-2 (2021-07-24)
==========================

* Bugfix: Keyboard events should now be properly forwarded from the Host
via the VST2 API. Confirmed to work in Ardour 6.
* Bugfix: Version label is now wider with a smaller font.
* Change: Middle mouse button in fine adj area removes modulation amount.
* Change: Resized the window from 1400x700 to 1400x787 to fit into the
Full HD aspect.
* Project: Added two sub crates: jack\_standalone and vst2


0.1.0-alpha-1 (2021-07-23)
==========================

* Initial pre-release for testing purposes.
