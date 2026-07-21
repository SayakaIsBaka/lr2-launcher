#[derive(Copy, Clone)]
pub enum Type {
    Unknown = 0,
    LR2,
    OpenLR2
}

pub struct Game {
    pub typ: Type,
    pub version: String
}