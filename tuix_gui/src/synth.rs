use crate::jack::start_backend;
use hexosynth::init_hexosynth;

use hexodsp::*;

use std::sync::Arc;
use std::sync::Mutex;

pub fn start<F: FnMut(Arc<Mutex<Matrix>>)>(mut f: F) {
    let (matrix, node_exec) = init_hexosynth();
    let matrix = Arc::new(Mutex::new(matrix));

    start_backend(node_exec, move || { f(matrix.clone()) })
}
