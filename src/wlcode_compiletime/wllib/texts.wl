!@export about = "[t18:][f22:About]

HexoSynth is a modular synthesizer where the graph is
represented as hexagonal tile map. The 6 edges of each tile
are the ports of the nodes (aka modules). The top and left edges
are the input edges, and the bottom and right edges are the outputs.

ATTENTION: For help please take a look at the other tabs of this about
           screen at the top!

[t9:]-------------------------------

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


!@export help = $q°[t2:][f40:WARNING!]
[f40:This help text is]
[f40:from HexoSynth 2021]
[f40:It reflects an old workflow!] 
[c18f22:Parameter Knobs]

[t9:]
Parameter knobs have two areas where you can grab them:
* Center value label is the coarse area.
* Bottom value label: below center is the fine adjustment area.
  The fine adjustment area will highlight and display the
  raw signal value of the input parameter. This can be useful
  if you want to build modulators that reach exactly a certain value.

Parameter Knobs are greyed out when it's corresponding input
is connected to an output. That means, the parameter value is not
been used. You can still change it if you want though, it just wont
make a difference as long as the input is in use.

    Drag LMB Up/Down                - Adjust parameter.
    Drag RMB Up/Down                - Adjust parameter modulation amount.
    Hover over knob + Mouse Wheel   - Adjust parameter.
    Shift + Drag LMB Up/Down        - Fine adjust parameter.
    Shift + Drag RMB Up/Down        - Fine adjust parameter mod. amount.
    Ctrl  + Drag LMB Up/Down        - Disable parameter snap. (eg. for the
                                      detune parameter)
    Ctrl + Shift + Drag LMB Up/Down - Fine adjustment of parameters with
                                      disabled parameter snap.
    MMB                             - Reset Parameter to it's default value.
    MMB (Knob fine adj. area)       - Reset Parameter to it's default value
                                      and remove modulation amount.
    Hover over knob + Backspace     - Remove parameter modulation amount.
    Hover over knob + Delete        - Remove parameter modulation amount.
    Hover over Knob + Enter         - Open the direct value entry.
    or:                               Coarse adjustment area will edit the
    Ctrl + RMB                        denormalized value. Fine adjustment
                                      area will edit the normalized
                                      -1..1 or 0..1 signal value. Hit 'Esc'
                                      to exit the value entry without change.

Combining the fine adjustment areas with the Shift key allows a freedom
of 4 resolutions to adjust parameters.

LMB = Left Mouse Button, RMB = Right Mouse Button, MMB = Middle Mouse Button
Next page: Hex Grid


[c18f22:Hex Grid / Node Matrix]

The hex tile grid consists of so "cells", each cell can be
empty or contain a "node". A cell with a node has the following structure:

      _____________
     /    <i1>     \
    /<i2>  (L)  <o1>\      (L)           - Status Led
   /   <node type>   \     <i1> to <i3>  - 3 Input ports
   \  <instance id>  /     <o1> to <o3>  - 3 Output ports
    \<i3>       <o2>/      <node type>   - Type of the node
     \     <o3>    /       <instance id> - Instance ID of the node
      """""""""""""

The input ports correspond to the parameters of the node. You can assign
these to output ports of adjacent cells. A connection between cells
does only work or exist if there is an output port assigned and to the
adjacent cells edge a corresponding input port.

You can have multiple independent instances of a node without problems.
But you can also (linked) copy the instance of one cell to another with
a mouse gesture (LMB drag from empty non adjacent node). This means:

            ____
      _____/    \_____      Two (linked) copies of a "Sin" oscialltor
     / Sin \____/ Sin \     node with the same instance id (0).
     \  0  /    \  0  /     These are handles to the same oscillator node.
      """""\____/"""""

Linked copies make it possible to connect more than 3 inputs or output
of a node to other nodes.

            ____
      _____/    \_____      Two independent instances of a "Sin" oscillator.
     / Sin \____/ Sin \     One has the instance id 0 and the other 1.
     \  0  /    \  1  /     These are handles to different and independent
      """""\____/"""""      oscillator nodes.


[c18f22:Hex Grid Mouse Actions (Part 1)]

The most basic actions are:

    LMB click on empty cell     - Open node selector menu.
    RMB click on any cell       - Open context menu for the cell.
    MMB drag grid               - Pan the grid around
    Scrollwheel Down/Up         - Zoom the grid in/out

Apart from these basics, there are multiple differnt mouse drag
gestures to change the node graph layout of the node matrix in the hex grid.

Some gestures do things with so called "node clusters". A "node cluster" is
a tree of connected nodes.

LMB (Left Mouse Button) Drag Actions:

    (LMB) Create two connected nodes with default ports:
      ..... <-- Drag Action
     _^_  .       ___       LMB Drag from empty to adjacent empty cell lets
    /   \_._  => /XXX\___   you select two new nodes from the node selector
    \___/ v \    \_0_/YYY\  menu. And connects these two nodes from their
        \___/        \_0_/  default output port to the default input port
                            of the other node.
                            (If you want to select the edges explicitly,
                             try dragging with RMB).

    (LMB) Create one new connected node with default ports:
      .....
     _^_  .       ___       LMB Drag from empty cell to adjacent node lets
    /   \_._  => /XXX\___   you select one new node from the node selector
    \___/YvY\    \_0_/YYY\  menu. And connects these two nodes from their
        \_0_/        \_0_/  default output ports to the default input port
                            of the other node.
                            (If you want to select the edges explicitly,
                             try dragging with RMB).

    (LMB) (Re)Connect Adjacent Cells:
       ......___            _____  Dragging a node to an adjacent node will
     __.__/ .   \     _____/     \ open the output/input port selection
    /  X  \_.___/ => / XXX \_____/ menu. You can connect two previously not
    \__0__/ vY  \    \__0_O/I Y  \ connected nodes with this or reconnect
          \__0__/          \__0__/ existing adjacent nodes.


[c18f22:Hex Grid Mouse Actions (Part 2)]

LMB (Left Mouse Button) Drag Actions:

    (LMB) Move Cluster:
      .....      .........                    LMB drag from cell with a node
     _^_  .     _^_     _._      ___     ___  to any empty cell moves a
    /XXX\_._   /XXX\___/ v \    /   \___/XXX\ whole cluster of nodes. Keep
    \_1_/ v \  \_1_/   \___/ => \___/   \_1_/ in mind: a cluster is a tree
    /YYY\___/  /YYY\___/            \___/YYY\ of connected nodes. This will
    \_2_/      \_2_/                    \_2_/ not move adjacent but
                                              unconnected nodes!

    (LMB) Create Linked Copy close to destination:
      .........
     _^_     _._      ___     ___  LMB drag from cell with a node to any
    /XXX\___/ . \    /XXX\___/XXX\ other non adjacent cell with a
    \_0_/   \_._/ => \_0_/   \_0_/ node (YYY 1) creates a linked but
        \___/YvY\        \___/YvY\ unconnected copy of the dragged from
            \_1_/            \_1_/ node (XXX 0).
                                   (If you want to create a new instance
                                    instead, try dragging with RMB).

    (LMB) Create Linked Copy at empty drag source:
      .........
     _._     _._      ___     ___  LMB drag from an empty non adjacent
    /XvX\___/ . \    /XXX\___/   \ cell to a node will create a linked
    \_1_/   \_._/ => \_1_/   \___/ copy of that node.
        \___/ ^ \        \___/XXX\ (If you want to create a new instance
            \___/            \_1_/ instead, try dragging with RMB).

RMB (Right Mouse Button) Drag Actions:

    (RMB) Create two connected nodes with explicit port selection menu:
      ..... <-- Drag Action
     _^_  .       ___       RMB Drag from empty to adjacent empty cell lets
    /   \_._  => /XXX\___   you select two new nodes from the node selector
    \___/ v \    \_0_/YYY\  menu. After selecting the two nodes, you have
        \___/        \_0_/  to explicitly choose which ports to connect
                            (unless there is only one input or output port).
                            (If you want to use the default inputs/outputs
                             try dragging with LMB).


[c18f22:Hex Grid Mouse Actions (Part 2)]

RMB (Right Mouse Button) Drag Actions:

    (RMB) Create one new connected node with explicit port selection menu:
      .....
     _^_  .       ___       RMB Drag from empty cell to adjacent node lets
    /   \_._  => /XXX\___   you select one new node from the node selector
    \___/YvY\    \_0_/YYY\  menu. And then requires you to explicitly
        \_0_/        \_0_/  select the input and output ports.
                            (If you want to use the default inputs/outputs
                             try dragging with LMB).

    (RMB) Move node:
      .....      .........
     _^_  .     _^_     _._      ___     ___  RMB drag from cell with a node
    /XXX\_._   /XXX\___/ v \    /   \___/XXX\ to any empty cell moves only
    \_1_/ v \  \_1_/   \___/ => \___/   \_1_/ the cell, ignoring any
    /YYY\___/  /YYY\___/        /YYY\___/   \ adjacent connected nodes.
    \_2_/      \_2_/            \_2_/   \___/

    (RMB) Create New Instance close to destination:
      .........
     _^_     _._      ___     ___  RMB drag from cell with a node to any
    /XXX\___/ . \    /XXX\___/XXX\ other non adjacent cell with a
    \_0_/   \_._/ => \_0_/   \_1_/ node (YYY 1) creates an unconnected new
        \___/YvY\        \___/YvY\ node instance with the same type as the
            \_1_/            \_1_/ drag source.
                                   (If you want to create a linked copy
                                    instead, try dragging with LMB).

    (RMB) Split a cluster
       .....
      _^_  .  ___      ___     ___  RMB drag between two connected nodes
     /XXX\_._/   \    /XXX\___/   \ will split the cluster (tree of
     \_1_/YvY\___/ => \_1_/   \___/ connected nodes) at that point and
     /   \_2_/   \    /   \___/YYY\ will make space for inserting a
     \___/   \___/    \___/   \_2_/ new node into that cluster.


[c18f22:Hex Grid Mouse Actions (Part 3)]

RMB (Right Mouse Button) Drag Actions:

    (RMB) Create a New Instance at empty drag source:
      .........
     _._     _._      ___     ___  RMB drag from an empty non adjacent
    /XvX\___/ . \    /XXX\___/   \ cell to a node will create a new
    \_1_/   \_._/ => \_1_/   \___/ node instance of the type of the
        \___/ ^ \        \___/XXX\ drag destination node.
            \___/            \_2_/ (If you want to create a new instance
                                   instead, try dragging with RMB).
°;

!@export tracker = $q°[t18:][f22:]Tracker / Pattern Editor Keyboard Shortcuts

[t18:]* Normal Mode
[t9:]
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

[t18:]* Edit Mode
[t9:]
    [c14:Up/Down/Left/Right] - Move Cursor

    [c14:'.']                - Enter most recently entered value
                               and advance one edit step.
    [c14:',']                - Remember the current cell value as most recently
                               used value and advance one edit step.
                               Useful for copying a value and paste it with [c14:'.'].
    Note Column  :    Note entering via keyboard "like Renoise".
    Other Columns:
        [c14:'0'-'9', 'a'-'f'] - Enter value in hex digits
        [c14:'s']              - Set to 000
        [c14:'g']              - Set to FFF
°;

!@export picker = $q°[t18:][f22:]Node Picker

The most precise way of placing a new node on the matrix can be achived
by dragging:

    [c14:Left Mouse Button Drag] - Dragging a button from the node picker to
                                   a matrix cell will place it.

If you click a button in the node picker and you have no cell in the matrix
selected, a new node will be placed close to the center of the visible hex matrix.

If a matrix cell is selected, following can be done:

    [c14:Left Mouse Button (LMB)]  - Add new node to the [c02:output] of the
                                     currently selected cell.
    [c14:Right Mouse Button (RMB)] - Add new node to the [c02:input] of the
                                     currently selected cell.

The picker has following categories:

* [c11:Osc]     - Sound generation sources and audio rate oscillators
* [c11:Mod]     - Modulation sources, such as LFOs, envelopes and sequencers
* [c11:NtoM]    - Signal routing nodes, such as mixers, faders or selectors
* [c11:Signal]  - Audio rate signal filters and effects
* [c11:Ctrl]    - Control signal modules, such as quantizers and range mappers
* [c11:IOUtil]  - Utility modules for HexoSynth, such as external inputs/outputs
            and FbWr/FbRd for creating feedback in the matrix.
°;
