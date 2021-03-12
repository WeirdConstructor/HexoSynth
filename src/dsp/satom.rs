use hexotk::Atom;

#[derive(Debug, Clone)]
pub enum SAtom {
    Str(String),
    MicroSample([f32; 8]),
    AudioSample((String, std::sync::Arc<Vec<f32>>)),
    Setting(i64),
    Param(f32),
}

impl SAtom {
    pub fn str(s: &str)         -> Self { SAtom::Str(s.to_string()) }
    pub fn setting(s: i64)      -> Self { SAtom::Setting(s) }
    pub fn param(p: f32)        -> Self { SAtom::Param(p) }
    pub fn micro(m: &[f32; 8])  -> Self { SAtom::MicroSample(*m) }
    pub fn audio(s: &str, m: std::sync::Arc<Vec<f32>>) -> Self {
        SAtom::AudioSample((s.to_string(), m))
    }

    pub fn default_of(&self) -> Self {
        match self {
            SAtom::Str(_)         => SAtom::Str("".to_string()),
            SAtom::MicroSample(_) => SAtom::MicroSample([0.0; 8]),
            SAtom::AudioSample(_) => SAtom::AudioSample(("".to_string(), std::sync::Arc::new(vec![]))),
            SAtom::Setting(_)     => SAtom::Setting(0),
            SAtom::Param(_)       => SAtom::Param(0.0),
        }
    }

    pub fn is_continous(&self) -> bool {
        if let SAtom::Param(_) = self { true }
        else { false }
    }

    pub fn i(&self) -> i64 {
        match self {
            SAtom::Setting(i) => *i,
            SAtom::Param(i)   => *i as i64,
            _                => 0,
        }
    }

    pub fn f(&self) -> f32 {
        match self {
            SAtom::Setting(i) => *i as f32,
            SAtom::Param(i)   => *i,
            _                => 0.0,
        }
    }
}

impl From<f32> for SAtom {
    fn from(n: f32) -> Self { SAtom::Param(n) }
}

impl From<Atom> for SAtom {
    fn from(n: Atom) -> Self {
        match n {
            Atom::Str(s)         => SAtom::Str(s),
            Atom::MicroSample(s) => SAtom::MicroSample(s),
            Atom::AudioSample(s) => SAtom::AudioSample(s),
            Atom::Setting(s)     => SAtom::Setting(s),
            Atom::Param(s)       => SAtom::Param(s),
        }
    }
}
