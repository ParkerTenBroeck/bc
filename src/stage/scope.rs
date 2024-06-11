use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Hash)]
pub enum Vis {
    Pub,
    Priv,
}

#[derive(Clone, Debug, Hash)]
pub struct Scope<'a> {
    pub vis: Vis,
    pub parts: Vec<ScopePart<'a>>,
}

#[derive(Clone, Debug, Hash)]
pub enum ScopePart<'a> {
    Module(Cow<'a, str>),
    Function(Cow<'a, str>),
}
