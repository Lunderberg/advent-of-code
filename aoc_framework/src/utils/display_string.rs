use std::fmt::{Debug, Error, Formatter};

pub struct DisplayString(String);

impl Debug for DisplayString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let str = &self.0;
        write!(f, "{str}")
    }
}

impl From<String> for DisplayString {
    fn from(str: String) -> Self {
        Self(str)
    }
}
