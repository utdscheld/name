use std::ops::{Add, BitAnd, BitOr, BitXor, Sub, BitAndAssign};

use std::fmt;

#[derive(Clone, Copy)]
pub struct U5 {
    pub value: u8,
}

pub fn mask5(v: u8) -> u8 {
    v & 0b11111
}

impl U5 {
    pub fn new(v: u8) -> Result<U5, &'static str> {
        let masked_value = mask5(v);
        if masked_value != v { 
            Err("Tried to place a value larger than 31 into a U5")
        } else {
            Ok(U5 { value: mask5(v) })
        }
        
    }

    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Add for U5 {
    type Output = Result<Self, &'static str>;

    fn add(self, other: Self) -> Self::Output {
        let result = self.value() + other.value();
        U5::new(result)
    }
}

impl Sub for U5 {
    type Output = Result<Self, &'static str>;

    fn sub(self, other: Self) -> Self::Output {
        let result = self.value().wrapping_sub(other.value());
        U5::new(result)
    }
}

impl BitAnd for U5 {
    type Output = Result<Self, &'static str>;

    fn bitand(self, other: Self) -> Self::Output {
        let result = self.value() & other.value();
        U5::new(result)
    }
}

impl BitOr for U5 {
    type Output = Result<Self, &'static str>;

    fn bitor(self, other: Self) -> Self::Output {
        let result = self.value() | other.value();
        U5::new(result)
    }
}

impl BitXor for U5 {
    type Output = Result<Self, &'static str>;

    fn bitxor(self, other: Self) -> Self::Output {
        let result: u8 = self.value() ^ other.value();
        U5::new(result)
    }
}

// Implement Debug to enable pretty printing with println! and debug_assert!
impl std::fmt::Debug for U5 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "U5({})", self.value)
    }
}

// Implement PartialEq and Eq for comparison
impl PartialEq for U5 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl fmt::LowerHex for U5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}", self.value())
    }
}

impl From<U5> for u32 {
    fn from(u5: U5) -> u32 {
        u5.value().into()
    }
}

impl From<U5> for u16 {
    fn from(u5: U5) -> u16 {
        u5.value().into()
    }
}

impl BitAndAssign for U5 {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self{value: self.value & rhs.value}
    }
}
