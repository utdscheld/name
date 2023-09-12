use std::ops::{Add, BitAnd, BitOr, BitXor, Sub, BitAndAssign, Shl};

use std::fmt;

#[derive(Clone, Copy)]
pub struct U6 {
    pub value: u8,
}

pub fn mask6(v: u8) -> u8 {
    v & 0b111111
}

impl U6 {
    pub fn new(v: u8) -> U6 {
        U6 { value: mask6(v) }
    }

    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Add for U6 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let result = self.value() + other.value();
        U6::new(result)
    }
}

impl Sub for U6 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let result = self.value().wrapping_sub(other.value());
        U6::new(result)
    }
}

impl BitAnd for U6 {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        let result = self.value() & other.value();
        U6::new(result)
    }
}

impl BitOr for U6 {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        let result = self.value() | other.value();
        U6::new(result)
    }
}

impl BitXor for U6 {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self::Output {
        let result: u8 = self.value() ^ other.value();
        U6::new(result)
    }
}

impl Shl for U6 {
    type Output = Self;

    fn shl(self, other: Self) -> Self::Output {
        let result: u8 = self.value() << other.value();
        U6::new(result)
    }
}

// Implement Debug to enable pretty printing with println! and debug_assert!
impl std::fmt::Debug for U6 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "U6({})", self.value)
    }
}

// Implement PartialEq and Eq for comparison
impl PartialEq for U6 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl fmt::LowerHex for U6 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}", self.value())
    }
}

impl From<U6> for u32 {
    fn from(u5: U6) -> u32 {
        u5.value().into()
    }
}

impl BitAndAssign for U6 {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self{value: self.value & rhs.value}
    }
}
