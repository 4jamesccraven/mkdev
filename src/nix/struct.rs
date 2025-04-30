use super::error::Error;
use super::ser::Serializer;

use serde::{ser, Serialize};

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.output.ends_with("{") {
            self.output += "\n";
        }

        let mut key_serializer = Serializer {
            output: String::new(),
            pending_key: None,
            indent_depth: self.indent_depth,
        };

        let mut val_serializer = Serializer {
            output: String::new(),
            pending_key: None,
            indent_depth: self.indent_depth,
        };

        value.serialize(&mut val_serializer)?;
        let val = val_serializer.output;

        if val == "NULL" {
            return Ok(());
        }

        self.indent();
        key.serialize(&mut key_serializer)?;
        let mut base_key = key_serializer.output;

        // TODO: Add cases where keys need to be escaped
        if true {
            let try_strip = base_key
                .strip_prefix("\"")
                .and_then(|s| s.strip_suffix("\""));

            match try_strip {
                Some(str) => base_key = str.to_string(),
                None => {}
            }
        }

        self.output += &base_key;
        self.output += " = ";
        self.output += &val;
        self.output += ";\n";

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.indent_depth -= 1;
        self.indent();
        self.output += "}";
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.output += " ";

        key.serialize(&mut **self)?;
        self.output += " = { ";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output += "}; }";
        Ok(())
    }
}
