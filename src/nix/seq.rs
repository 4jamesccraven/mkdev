use super::error::Error;
use super::ser::Serializer;

use serde::{ser, Serialize};

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.output += "\n";
        self.indent();
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.indent_depth -= 1;
        self.output += "\n";
        self.indent();
        self.output += "]";
        Ok(())
    }
}
