use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Move {
    off: bool,
    talon: bool,
    from: u8,
    to: u8,
    index: u8,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(index: {}, from: {}, to: {})",
            self.index, self.from, self.to
        )
    }
}

impl Move {
    pub fn regular(from: usize, to: usize, index: usize) -> Move {
        Move {
            talon: false,
            off: false,
            from: from as u8,
            to: to as u8,
            index: index as u8,
        }
    }
    pub fn from_talon(from: usize) -> Move {
        Move {
            talon: true,
            off: false,
            from: from as u8,
            to: 0,
            index: 0,
        }
    }
    pub fn off(from: usize, index: usize) -> Move {
        Move {
            talon: false,
            off: true,
            from: from as u8,
            to: 0,
            index: index as u8,
        }
    }
    pub fn invalid() -> Move {
        Move {
            talon: false,
            off: false,
            from: 11,
            to: 0,
            index: 0,
        }
    }

    pub fn from(&self) -> usize {
        self.from as usize
    }

    pub fn to(&self) -> usize {
        self.to as usize
    }

    pub fn index(&self) -> usize {
        self.index as usize
    }
    pub fn is_off(&self) -> bool {
        self.off
    }
    pub fn is_talon(&self) -> bool {
        self.talon
    }

    pub fn is_invalid(&self) -> bool {
        self.from > 10
    }
}
