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

fn deserialize_node_id(v: &Value, i1: usize, i2: usize)
    -> Result<NodeId, MatrixDeserError>
{
    let nid = NodeId::from_str(v[i1].as_str().unwrap_or("???"));
    if nid == NodeId::Nop {
        return Err(
            MatrixDeserError::UnknownNode(
                v[i1].as_str().unwrap_or("???").to_string()));
    }

    Ok(nid.to_instance(v[i2].as_i64().unwrap_or(0) as usize))
}

impl CellRepr {
    fn serialize(&self) -> Value {
        json!([
            self.node_id.name(),
            self.node_id.instance(),
            self.x,
            self.y,
            [self.inp[0], self.inp[1], self.inp[2]],
            [self.out[0], self.out[1], self.out[2]],
        ])
    }

    fn deserialize(v: &Value) -> Result<Self, MatrixDeserError> {
        Ok(Self {
            node_id: deserialize_node_id(v, 0, 1)?,
            x:       v[2].as_i64().unwrap_or(0) as usize,
            y:       v[3].as_i64().unwrap_or(0) as usize,
            inp: [
                v[4][0].as_i64().unwrap_or(-1) as i16,
                v[4][1].as_i64().unwrap_or(-1) as i16,
                v[4][2].as_i64().unwrap_or(-1) as i16,
            ],
            out: [
                v[5][0].as_i64().unwrap_or(-1) as i16,
                v[5][1].as_i64().unwrap_or(-1) as i16,
                v[5][2].as_i64().unwrap_or(-1) as i16,
            ],
        })
    }
}

#[derive(Debug, Clone)]
pub struct MatrixRepr {
    pub cells:   Vec<CellRepr>,
    pub params:  Vec<(ParamId, f32)>,
    pub atoms:   Vec<(ParamId, SAtom)>,
}

#[derive(Debug, Clone)]
pub enum MatrixDeserError {
    BadVersion,
    UnknownNode(String),
    UnknownParamId(String),
    Deserialization(String),
    InvalidAtom(String),
}

impl From<serde_json::Error> for MatrixDeserError {
    fn from(err: serde_json::Error) -> MatrixDeserError {
        MatrixDeserError::Deserialization(format!("{}", err))
    }
}

fn deserialize_atom(v: &Value) -> Result<SAtom, MatrixDeserError> {
    match v[0].as_str().unwrap_or("?") {
        "i" => {
            if let Some(v) = v[1].as_i64() { Ok(SAtom::setting(v)) }
            else { Err(MatrixDeserError::InvalidAtom(v.to_string())) }
        },
        "p" => {
            if let Some(v) = v[1].as_f64() { Ok(SAtom::param(v as f32)) }
            else { Err(MatrixDeserError::InvalidAtom(v.to_string())) }
        },
        "s" => {
            if let Some(v) = v[1].as_str() { Ok(SAtom::str(v)) }
            else { Err(MatrixDeserError::InvalidAtom(v.to_string())) }
        },
        "as" => {
            if let Some(v) = v[1].as_str() { Ok(SAtom::audio_unloaded(v)) }
            else { Err(MatrixDeserError::InvalidAtom(v.to_string())) }
        },
        "ms" => {
            let mut buf : [f32; 8] = [0.0; 8];

            for i in 0..8 {
                if let Some(v) = v[i + 1].as_f64() {
                    buf[i] = v as f32;
                } else {
                    return Err(MatrixDeserError::InvalidAtom(v.to_string()));
                }
            }

            Ok(SAtom::micro(&buf))
        },
        _ => Err(MatrixDeserError::InvalidAtom(v.to_string())),
    }
}

fn serialize_atom(atom: &SAtom) -> Value {
    match atom {
        SAtom::MicroSample(s) => json!(["ms",
            s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7],
        ]),
        SAtom::Str(s)              => json!(["s", s]),
        SAtom::AudioSample((s, _)) => json!(["as", s]),
        SAtom::Setting(i)          => json!(["i", i]),
        SAtom::Param(p)            => json!(["p", p]),
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

    pub fn write_to_file(&mut self, filepath: &str) -> std::io::Result<()> {
        use std::io::prelude::*;
        use std::fs::OpenOptions;

        let tmp_filepath = format!("{}~", filepath);

        let mut ser = self.serialize();
        ser.push('\n');

        let mut file =
            OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_filepath)?;
            file.write_all(ser.as_bytes())?;
            std::fs::rename(&tmp_filepath, &filepath)?;

            Ok(())
    }

    pub fn deserialize(s: &str) -> Result<MatrixRepr, MatrixDeserError> {
        let v : Value = serde_json::from_str(s)?;

        if let Some(version) = v.get("VERSION") {
            let version : i64 = version.as_i64().unwrap_or(0);

            if version != 1 {
                return Err(MatrixDeserError::BadVersion);
            }
        }

        let mut m = MatrixRepr::empty();

        let cells = &v["cells"];
        if let Value::Array(cells) = cells {
            for c in cells.iter() {
                m.cells.push(CellRepr::deserialize(c)?);
            }
        }

        let params = &v["params"];
        if let Value::Array(params) = params {
            for v in params.iter() {
                let node_id = deserialize_node_id(&v, 0, 1)?;
                let param_id = node_id.inp_param(v[2].as_str().unwrap_or(""));

                if let Some(param_id) = param_id {
                    m.params.push(
                        (param_id, v[3].as_f64().unwrap_or(0.0) as f32));
                } else {
                    return Err(
                        MatrixDeserError::UnknownParamId(v.to_string()));
                }
            }
        }

        let atoms = &v["atoms"];
        if let Value::Array(atoms) = atoms {
            for v in atoms.iter() {
                let node_id = deserialize_node_id(&v, 0, 1)?;
                let param_id = node_id.inp_param(v[2].as_str().unwrap_or(""));

                if let Some(param_id) = param_id {
                    m.atoms.push((param_id, deserialize_atom(&v[3])?))
                } else {
                    return Err(
                        MatrixDeserError::UnknownParamId(v.to_string()));
                }
            }
        }


        Ok(m)
    }

    pub fn serialize(&mut self) -> String {
        let mut v = json!({
            "VERSION": 1,
        });

        self.params.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        self.atoms.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut params = json!([]);
        if let Value::Array(params) = &mut params {
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

        let mut atoms = json!([]);
        if let Value::Array(atoms) = &mut atoms {
            for (p, v) in self.atoms.iter() {
                atoms.push(
                    json!([
                        p.node_id().name(),
                        p.node_id().instance(),
                        p.name(),
                        serialize_atom(v),
                    ]));
            }
        }

        v["atoms"] = atoms;

        let mut cells = json!([]);
        if let Value::Array(cells) = &mut cells {
            for cell in self.cells.iter() {
                cells.push(cell.serialize());
            }
        }

        v["cells"] = cells;

        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::matrix::{Matrix, Cell};

    #[test]
    fn check_empty_repr_serialization() {
        let mut matrix_repr = MatrixRepr::empty();

        let s = matrix_repr.serialize();

        assert_eq!(s,
            "{\"VERSION\":1,\"atoms\":[],\"cells\":[],\"params\":[]}");
        assert!(MatrixRepr::deserialize(&s).is_ok());
    }


    #[test]
    fn check_repr_serialization() {
        use crate::nodes::new_node_engine;

        let (node_conf, mut _node_exec) = new_node_engine();
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

        assert_eq!(s,
          "{\"VERSION\":1,\"atoms\":[[\"out\",0,\"mono\",[\"i\",0]]]\
          ,\"cells\":[[\"sin\",2,0,0,[-1,-1,-1],[-1,0,-1]],\
          [\"out\",0,1,0,[-1,0,-1],[-1,-1,0]]],\
          \"params\":[\
          [\"out\",0,\"ch1\",0.0],[\"out\",0,\"ch2\",0.0],\
          [\"sin\",0,\"freq\",0.0],[\"sin\",1,\"freq\",0.0],\
          [\"sin\",2,\"freq\",-0.10000000149011612]]}");

        let mut mr2 = MatrixRepr::deserialize(&s).unwrap();

        let s2 = mr2.serialize();

        assert_eq!(s, s2);
    }

    #[test]
    fn check_atom_repr() {
        let v = serialize_atom(&SAtom::str("foo"));
        assert_eq!(v.to_string(), "[\"s\",\"foo\"]");
        let s = serialize_atom(&deserialize_atom(&v).unwrap()).to_string();
        assert_eq!(s, v.to_string());

        let v = serialize_atom(&SAtom::setting(1337));
        assert_eq!(v.to_string(), "[\"i\",1337]");
        let s = serialize_atom(&deserialize_atom(&v).unwrap()).to_string();
        assert_eq!(s, v.to_string());

        let v = serialize_atom(&SAtom::param(1.0));
        assert_eq!(v.to_string(), "[\"p\",1.0]");
        let s = serialize_atom(&deserialize_atom(&v).unwrap()).to_string();
        assert_eq!(s, v.to_string());

        let v =
            serialize_atom(
                &SAtom::micro(&[1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0]));
        assert_eq!(v.to_string(), "[\"ms\",1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0]");
        let s = serialize_atom(&deserialize_atom(&v).unwrap()).to_string();
        assert_eq!(s, v.to_string());

        let v =
            serialize_atom(
                &SAtom::audio(
                    "lol.wav",
                    std::sync::Arc::new(vec![1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0])));
        assert_eq!(v.to_string(), "[\"as\",\"lol.wav\"]");
        let s = serialize_atom(&deserialize_atom(&v).unwrap()).to_string();
        assert_eq!(s, v.to_string());
    }

    #[test]
    fn check_cell_repr() {
        let cell =
            Cell::empty(NodeId::Out(2))
            .input(Some(2), Some(0), Some(3))
            .out(Some(11), Some(4), Some(1));
        let cr = cell.to_repr();

        let s = cr.serialize().to_string();

        let v : Value = serde_json::from_str(&s).unwrap();
        let cr2 = CellRepr::deserialize(&v).unwrap();

        let s2 = cr2.serialize().to_string();
        assert_eq!(s, s2);
    }

    #[test]
    fn check_file_repr() {
        {
            use crate::nodes::new_node_engine;

            let (node_conf, mut _node_exec) = new_node_engine();
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

            mr.write_to_file("hexosynth_test_patch.hxy").unwrap();
        }

//        {
//            use crate::nodes::new_node_engine;
//
//            let mut mr = MatrixRepr::empty();
//
//            mr.read_from_file("hexosynth_test_patch.hxy").unwrap();
//
//            let (node_conf, mut _node_exec) = new_node_engine();
//            let mut matrix = Matrix::new(node_conf, 3, 3);
//
//            matrix.from_repr(mr);
//            matrix.sync();
//
//            let mut mr = matrix.to_repr();
//            let s = mr.serialize().to_string();
//
//            assert_eq!(s, "");
//        }

    }
}
