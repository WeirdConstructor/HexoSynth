use crate::nodes::NodeConfigurator;
use crate::dsp::{NodeId, NodeInfoHolder, NodeInfo};

pub struct Matrix {
    info_holder: NodeInfoHolder,
    config:      NodeConfigurator,
}

pub struct Cell {
    node_id: NodeId,
    out1: Option<u8>,
    out2: Option<u8>,
    out3: Option<u8>,
    in1:  Option<u8>,
    in2:  Option<u8>,
    in3:  Option<u8>,
}

impl Cell {
    pub fn empty(node_id: NodeId) -> Self {
        Self {
            node_id,
            out1: None,
            out2: None,
            out3: None,
            in1: None,
            in2: None,
            in3: None,
        }
    }
}

impl Matrix {
    pub fn new(config: NodeConfigurator) -> Self {
        Self {
            info_holder: NodeInfoHolder::new(),
            config,
        }
    }

    pub fn place(&mut self, x: usize, y: usize, cell: Cell) {
    }

    pub fn sync(&mut self) {
        // For all cells without an NodeInstance, let NodeConfigurator create one
    }
}


/*

Design of the highlevel Matrix API:

- NodeInfo (belongs to nothing, is the root of knowledge)
  - name
  - GUI type (Default, ModFunction, LFO+MF, 3xLFO+MF, ADSR+MF, ...)
  - output ports: number and name
  - input ports: number and name
    - input parameter range
    - input parameter normalization/denormalization
    - input parameter formatting

- NodeCollection (changes are transmitted to the backend!)
    - List all possible node types (NodeInfo) "Sin", "Amp", "Out"
    - List existing instances "Sin 1", "Sin 2", ... with their NodeInfo
        => NodeInstance
    - Instanciate new nodes (they get a global identifier)
    - Update an input parameter by Instance ID and input index.

- Matrix (has a NodeCollection)
    (changes are transmitted to the backend)
    - place instanciated nodes somewhere with an input/output configuration
      (=> Define a Cell, which comes with 3 in and 3 out indices)
    - clear a cell of the matrix
    - get a cell of the matrix
    - make a selection of cells
    - copy that selection
    - paste a selection to somewhere else

- Query Node instance state InstanceState:
    - frontend parameter values (knobs)
    - output state
      - the backend should just provide a triple buffer with this
        and the NodeCollection somehow makes the output ports
        accessible by instance

- Cells (belong to Matrix)
    - Come with an instance ID
    - Get the instance name
    - Get the name of the assigned output and input ports
    - Flag if the cell is selected
    - Assign new edge input/outputs


What the GUI needs:

- ?

I still need to decide how to refer to node instances:

- by global unique ID => how to recreate these IDs from a saved repr?
- By NodeType + Index
  - More generic in my gut feeling
  - Problem: NodeCollection needs to be able to check
             which internal index this node resides in.
             For this a linear scan over all nodes is necessary.
             But there are only ~100 nodes, so this should not
             take too much time!
  - Invariant: Don't delete nodes. Only delete them on a manual user
               initiated "garbage collect" which renames the nodes in the matrix.


*/
