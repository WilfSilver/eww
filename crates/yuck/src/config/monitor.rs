use std::{convert::Infallible, fmt, str};

use serde::{Deserialize, Serialize};
use simplexpr::dynval::{ConversionError, DynVal};

/// The type of the identifier used to select a monitor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorIdentifier {
    Numeric(i32),
    Name(String),
}

impl MonitorIdentifier {
    pub fn from_dynval(val: &DynVal) -> Result<Self, ConversionError> {
        match val.as_i32() {
            Ok(x) => Ok(MonitorIdentifier::Numeric(x)),
            Err(_) => Ok(MonitorIdentifier::Name(val.as_string()?)),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Numeric(_))
    }
}

impl fmt::Display for MonitorIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numeric(n) => write!(f, "{}", n),
            Self::Name(n) => write!(f, "{}", n),
        }
    }
}

impl str::FromStr for MonitorIdentifier {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i32>() {
            Ok(n) => Ok(Self::Numeric(n)),
            Err(_) => Ok(Self::Name(s.to_owned())),
        }
    }
}
