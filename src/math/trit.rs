use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trit {
    Neg,
    Zero,
    Pos,
}

impl Display for Trit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Trit::Neg => "T",
            Trit::Zero => "0",
            Trit::Pos => "1",
        };
        f.write_str(s)
    }
}

impl Default for Trit {
    fn default() -> Self {
        Self::Zero
    }
}

impl From<bool> for Trit {
    fn from(val: bool) -> Self {
        if val {
            Self::Pos
        } else {
            Self::Neg
        }
    }
}

impl Into<Option<bool>> for Trit {
    fn into(self) -> Option<bool> {
        match self {
            Trit::Neg => Some(false),
            Trit::Zero => None,
            Trit::Pos => Some(true),
        }
    }
}
