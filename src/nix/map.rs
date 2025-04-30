use super::error::Error;
use super::ser::Serializer;

use serde::{ser, Serialize};

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut key_serializer = Serializer {
            output: String::new(),
            pending_key: None,
            indent_depth: self.indent_depth,
        };
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

        self.pending_key = Some(base_key);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut value_serializer = Serializer {
            output: String::new(),
            pending_key: None,
            indent_depth: self.indent_depth,
        };

        value.serialize(&mut value_serializer)?;
        let val = value_serializer.output;

        if val == "NULL" {
            self.pending_key = None;
            return Ok(());
        }

        self.output += "\n";

        self.indent();
        let key = self.pending_key.take().expect("Value without key.");
        self.output += &format!("{} = {};", key, val);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.indent_depth -= 1;
        self.output += "\n";
        self.indent();
        self.output += "}";
        Ok(())
    }
}
