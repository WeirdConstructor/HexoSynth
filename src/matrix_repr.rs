use crate::dsp::{NodeId, ParamId, SAtom};
use serde_json::{Value, json};

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

    pub fn serialize(&self) -> String {
        let v = json!({
            "VERSION": 1,
        });

        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_empty_repr_serialization() {
        let matrix_repr = MatrixRepr::empty();

        let s = matrix_repr.serialize();

        assert_eq!(s, "{\"VERSION\":1}");
        assert!(MatrixRepr::deserialize(&s).is_ok());
    }
}
