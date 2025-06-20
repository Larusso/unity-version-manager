use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DigitalUnit {
    Byte,
    Kilobyte,
    Megabyte,
    Gigabyte,
}

impl DigitalUnit {
    fn to_bytes(&self) -> f64 {
        match self {
            DigitalUnit::Byte => 1.0,
            DigitalUnit::Kilobyte => 1_000.0,
            DigitalUnit::Megabyte => 1_000_000.0,
            DigitalUnit::Gigabyte => 1_000_000_000.0,
        }
    }
}

impl fmt::Display for DigitalUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DigitalUnit::*;
        let s = match self {
            Byte => "byte",
            Kilobyte => "kilobyte",
            Megabyte => "megabyte",
            Gigabyte => "gigabyte",
        };
        write!(f, "{}", s)
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy, Deserialize, Serialize)]
pub struct DigitalValue {
    pub value: f64,
    unit: DigitalUnit,
}

impl DigitalValue {
    pub fn to_bytes(self) -> DigitalValue {
        self.convert_to(DigitalUnit::Byte)
    }
    pub fn to_kilobytes(self) -> DigitalValue {
        self.convert_to(DigitalUnit::Kilobyte)
    }
    pub fn to_megabytes(self) -> DigitalValue {
        self.convert_to(DigitalUnit::Megabyte)
    }
    pub fn to_gigabytes(self) -> DigitalValue {
        self.convert_to(DigitalUnit::Gigabyte)
    }

    fn convert_to(self, target_unit: DigitalUnit) -> DigitalValue {
        let bytes = self.value * self.unit.to_bytes();
        let converted_value = bytes / target_unit.to_bytes();
        DigitalValue {
            value: converted_value,
            unit: target_unit,
        }
    }
}

impl Eq for DigitalValue {}

impl Ord for DigitalValue {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_bytes = self.value * self.unit.to_bytes();
        let other_bytes = other.value * other.unit.to_bytes();

        // Compare by converted values first
        match self_bytes.total_cmp(&other_bytes) {
            Ordering::Equal => self.unit.cmp(&other.unit),
            other => other,
        }
    }
}

impl Hash for DigitalValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.to_bits().hash(state);
        self.unit.hash(state);
    }
}

impl fmt::Display for DigitalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.unit)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BytesWrapper(f64);

impl PartialEq for BytesWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for BytesWrapper {}

impl PartialOrd for BytesWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for BytesWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for BytesWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl BytesWrapper {
    fn new(value: f64) -> Self {
        BytesWrapper(value)
    }
}

impl fmt::Display for BytesWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for BytesWrapper {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Size {
    Bytes(BytesWrapper),
    DigitalValue(DigitalValue),
}

impl Size {
    pub fn as_bytes_representation(&mut self) {
        match self {
            Size::Bytes(_) => {}
            Size::DigitalValue(d) => {
                *self = Size::Bytes(BytesWrapper::new(d.to_bytes().value))
            }
        }
    }

    pub fn to_bytes(&self) -> f64 {
        match self {
            Size::Bytes(b) => *b.deref(),
            Size::DigitalValue(d) => {
                d.to_bytes().value
            }
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Size::*;
        match self {
            Bytes(bytes) => {
                write!(f, "{} byte", bytes)
            }
            DigitalValue(d) => {
                write!(f, "{} {}", d.value, d.unit)
            }
        }
    }
}
