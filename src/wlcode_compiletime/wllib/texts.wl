!@export about = "# About

HexoSynth is a modular synthesizer where the graph is
represented as hexagonal tile map. The 6 edges of each tile
are the ports of the nodes (aka modules). The top and left edges
are the input edges, and the bottom and right edges are the outputs.

------------------------------------------------------------------

# Authors, Contributors, Credits

- *Dimas Leenman* (aka *Skythedragon*)
  - Author of the `FormFM` node
- *Weird Constructor*
  - Engine, GUI, many nodes

------------------------------------------------------------------

# License

HexoSynth Modular Synthesizer (Plugin or Standalone Application)
Copyright (C) 2021-2022  Weird Constructor <weirdconstructor@gmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
";


!@export help = $q°# HexoSynth - Modular Synthesizer

HexoSynth is a modular synthesizer plugin (VST3, CLAP). Like
those encountered in projects like VCVRack or Bitwig's Grid.

The core idea is having a hexagonal tile map for laying out
module instances and connect them at the edges to route audio signals
and control signals to inputs of other modules.

A goal is to provide a simple wireless environment to build sound effects,
synthesizers or whole generative music patches from predefined modules.

## GUI Overview

Here is a rough overview of the GUI:

![](res/hexosynth_gui_overview.png?380)

I hope most things are self explanatory. Most panels come with their own _"?"
help button_, and I suggest clicking on them to learn about the details.

The following help will focus mostly on the usage of the *Cell Matrix*, which
is the main interface to the modular synthesizer.

## Introduction

First let me explain the interface concept of HexoSynth.
Lets define a few terms first, so we can talk about them:

![](res/cell_concept.png?300)

- *Cell* - This is a hex tile cell. It can hold a so called *Node Instance*.
- *Node* - A DSP node is what is called a _module_ in other modular synthesizers.
You build a DSP graph of `Node Instances` in HexoSynth.
- *Node Type* - The _type_ or _kind_ of a node here refers to what the node
is actually doing. Like `Sin` which is a sine oscillator, or `Out` which represents
the synthesizer output. Or `SFilter` (Filter node), `Amp` (Amplifier) or `Mix3` (3 Channel Mixer).
- *Node Instance* - A node instance is an actual instanciation of a _node type_.
You can have multiple instances. You tell them apart using the *Instance IDs*.
- *Instance ID* - This ID tells the instances of the same _node type_ apart.
`Sin 0` is the `Sin` node instance with ID **0**. Where `Sin 1` is the `Sin` node
instance with ID **1**. They are completely different DSP nodes, each having their
own set of parameters and internal state.
- *Selected Cell* - The highlighted cell is the selected cell. If you select
a cell with a node instance, you can adjust it's parameters and see the input/outputs
of that cell in the signal monitor (the 6 scopes in the bottom left of the GUI).

You can have multiple *Cells* refer to the same *Node Instance*. The following
image illustrates that:

![](res/node_instances.png?300)

One *Cell* offers 3 *inputs* and 3 *outputs* to a *node instance*. The *inputs*
are also called ~~parameters~~ sometimes. Most ~~parameters~~ of a `Node` are also
exposed as *inputs*.

![](res/node_inputs_outputs.png?300)

Connecting inputs and outputs is done across the edges of a *Cell*. That is also
illustrated in the following image:

![](res/node_overview.png?300)

The top left *edges* of a cell show the *input name*, and the bottom right *edges*
show the *output name*.

°;

!@export tracker = $q°# Tracker / Pattern Editor Keyboard Shortcuts

## Normal Mode

```wichtext
    [c14:Return]             - Enter Edit Mode
    [c14:Escape]             - Exit Edit Mode

    [c14:Home]               - Cursor to first row
    [c14:End]                - Cursor to last row (within edit step)
    [c14:Page Up]            - Cursor up by 2 edit steps
    [c14:Page Down]          - Cursor down by 2 edit steps
    [c14:Up/Down/Left/Right] - Move Cursor
    [c14:'f']                - Toggle cursor follow phase bar

    [c14:Del]                - Delete value in cell at cursor
    [c14:'+' / '-']          - In-/Decrease note enter mode octave
    [c14:'*' / '/' (Keypad)] - In-/Decrease edit step by 1
    [c14:'r']                - Enter new pattern rows / length mode
    [c14:'e']                - Enter new edit step mode
    [c14:'o']                - Enter octave mode
    [c14:'c']                - Change column type mode
    [c14:'d']                - Delete col/row/step mode

    [c14:Shift + PgUp]   - (+ 0x100) Increase 1st nibble of value under cursor
    [c14:Shift + PgDown] - (- 0x100) Decrease 1st nibble of value under cursor
    [c14:Shift + Up]     - (+ 0x010) Increase 2nd nibble of value under cursor
    [c14:Shift + Down]   - (- 0x010) Decrease 2nd nibble of value under cursor
    [c14:Shift + Right]  - (+ 0x001) Increase 3rd nibble of value under cursor
    [c14:Shift + Left]   - (- 0x001) Decrease 3rd nibble of value under cursor
```

## Edit Mode

```wichtext
    [c14:Up/Down/Left/Right] - Move Cursor

    [c14:'.']                - Enter most recently entered value
                         and advance one edit step.
    [c14:',']                - Remember the current cell value as most recently
                         used value and advance one edit step.
                         Useful for copying a value and paste it with [c14:'.'].

    Note Column: Note entering via keyboard "like Renoise".

    Other Columns:
        [c14:'0'-'9', 'a'-'f'] - Enter value in hex digits
        [c14:'s']              - Set to 000
        [c14:'g']              - Set to FFF
```
°;

!@export picker = $q°# Node Picker

The most precise way of placing a new node on the matrix can be achived
by dragging:

    Left Mouse Button Drag - Dragging a button from the node picker to
                             a matrix cell will place it.

If you click a button in the node picker and you have no cell in the matrix
selected, a new node will be placed close to the center of the visible hex matrix.

If a matrix cell is selected, following can be done:

    Left Mouse Button (LMB)  - Add new node to the [c02:output] of the
                               currently selected cell.
    Right Mouse Button (RMB) - Add new node to the [c02:input] of the
                               currently selected cell.

The picker has following categories:

* `Osc` - Sound generation sources and audio rate oscillators
* `Mod` - Modulation sources, such as LFOs, envelopes and sequencers
* `NtoM` - Signal routing nodes, such as mixers, faders or selectors
* `Signal` - Audio rate signal filters and effects
* `Ctrl` - Control signal modules, such as quantizers and range mappers
* `IOUtil` - Utility modules for HexoSynth, such as external inputs/outputs
             and FbWr/FbRd for creating feedback in the matrix.
°;

!@export ext_param = $q°# External Parameters

With the `ExtA`, `ExtB` to `ExtF` nodes (in the `IOUtil` tab in the node picker),
you get access to the so called "external parameters". These are parameters
exposed to the **VST3/CLAP** host (the DAW usually). In the nih-plug standalone
version of HexoSynth these are not accessible from the outside.

The **"External Parameters"** panel in HexoSynth allows you to change these
parameters. That means you get a panel of 24 parameters you can freely use
to control your patch.

*Please note*: The parameter values are *NOT SAVED* by HexoSynth'
integrated 'Save' / 'Load' patch functionality. These parameters are to be
saved and controlled by the VST3/CLAP Plugin host (the DAW) currently.

Unfortunately you can't rename these parameters at this point. So you will
kind of need to remember what ~~ExtC3~~ is mapped to in your patch. I hope you
can manage with this.
°;
