use crate::dsp::{NodeId, ParamId, SAtom};
use serde_json::{Value, json};
use crate::matrix::{Matrix, Cell};

#[derive(Debug, Clone, Copy)]
pub struct CellRepr {
    pub node_id: NodeId,
    pub x:       usize,
    pub y:       usize,
    pub inp:     [i16; 3],
    pub out:     [i16; 3],
}

#[derive(Debug, Clone)]
pub struct MatrixRepr {
    pub cells:   Vec<CellRepr>,
    pub params:  Vec<(ParamId, f32)>,
    pub atoms:   Vec<(ParamId, SAtom)>,
}

#[derive(Debug, Clone)]
pub enum MatrixFormatError {
    BadVersion,
    Deserialization(String),
}

impl From<serde_json::Error> for MatrixFormatError {
    fn from(err: serde_json::Error) -> MatrixFormatError {
        MatrixFormatError::Deserialization(format!("{}", err))
    }
}

impl MatrixRepr {
    pub fn empty() -> Self {
        let cells  = vec![];
        let params = vec![];
        let atoms  = vec![];

        Self {
            cells,
            params,
            atoms,
        }
    }

    pub fn deserialize(s: &str) -> Result<MatrixRepr, MatrixFormatError> {
        let v : serde_json::Value = serde_json::from_str(s)?;

        if let Some(version) = v.get("VERSION") {
            let version : i64 = version.as_i64().unwrap_or(0);

            if version != 1 {
                return Err(MatrixFormatError::BadVersion);
            }
        }

        let m = MatrixRepr::empty();

        Ok(m)
    }

    pub fn serialize(&mut self) -> String {
        let mut v = json!({
            "VERSION": 1,
        });

        self.params.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut params = json!([]);
        if let serde_json::Value::Array(params) = &mut params {
            for (p, v) in self.params.iter() {
                params.push(
                    json!([
                        p.node_id().name(),
                        p.node_id().instance(),
                        p.name(),
                        v
                    ]));
            }
        }

        v["params"] = params;

        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_empty_repr_serialization() {
        let mut matrix_repr = MatrixRepr::empty();

        let s = matrix_repr.serialize();

        assert_eq!(s, "{\"VERSION\":1}");
        assert!(MatrixRepr::deserialize(&s).is_ok());
    }


    #[test]
    fn check_repr_serialization() {
        use crate::nodes::new_node_engine;

        let (node_conf, mut node_exec) = new_node_engine();
        let mut matrix = Matrix::new(node_conf, 3, 3);

        let sin = NodeId::Sin(2);

        matrix.place(0, 0,
            Cell::empty(sin)
            .out(None, Some(0), None));
        matrix.place(1, 0,
            Cell::empty(NodeId::Out(0))
            .input(None, Some(0), None)
            .out(None, None, Some(0)));
        matrix.sync();

        let freq_param = sin.inp_param("freq").unwrap();
        matrix.set_param(freq_param, SAtom::param(-0.1));

        let mut mr = matrix.to_repr();

        let s = mr.serialize();

        println!("FFF {:?}", mr);

        assert_eq!(s, "{ \"VERSION\":1}");

        assert!(MatrixRepr::deserialize(&s).is_ok());
    }


}
