use std::fmt;

use crate::common::color::*;

/// BlockType Enum
///
/// TODO: Add more block type
#[derive(Debug)]
pub enum BlockType {
    Solid { color: Color },
}

impl fmt::Display for BlockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockType::Solid { color } => write!(f, "<solid {color}>", color = color),
        }
    }
}

/// Block data(name and data)
#[derive(Debug)]
pub struct Block {
    /// Block name
    pub name: &'static str,
    /// Block data
    pub data: BlockType,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Block { name, data } = self;
        write!(f, "<{name}: {data}>", name = *name, data = data)
    }
}

#[allow(dead_code)]
pub mod constants {
    use super::*;

    const fn solid_block(name: &'static str, color: Color) -> Block {
        Block {
            name,
            data: BlockType::Solid { color },
        }
    }

    pub const GREEN_BLOCK: &'static Block = &solid_block("green", GREEN);
    pub const RED_BLOCK: &'static Block = &solid_block("red", RED);
    pub const BLUE_BLOCK: &'static Block = &solid_block("blue", BLUE);
    pub const PURPLE_BLOCK: &'static Block = &solid_block("purple", PURPLE);
    pub const YELLOW_BLOCK: &'static Block = &solid_block("yellow", YELLOW);
    pub const AQUA_BLOCK: &'static Block = &solid_block("aqua", AQUA);
}
