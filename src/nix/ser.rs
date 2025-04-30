use super::error::Error;

use serde::{ser, Serialize};

pub struct Serializer {
    pub output: String,
    pub pending_key: Option<String>,
    pub indent_depth: usize,
}

impl Serializer {
    pub fn indent(&mut self) {
        self.output += &"  ".repeat(self.indent_depth);
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let multiline = v.contains('\n') && v.len() >= 80;

        match multiline {
            true => {
                self.output += "''\n";
                self.indent_depth += 1;
                self.indent();
            }
            false => self.output += "\"",
        }

        let mut chars = v.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\'' => match chars.peek() {
                    Some(&'\'') if multiline => {
                        self.output += "''";
                    }
                    _ => self.output += "'",
                },
                '"' => match multiline {
                    true => self.output += "\"",
                    false => self.output += "\\\"",
                },
                '\\' => match multiline {
                    true => self.output += "\\",
                    false => self.output += "\\\\",
                },
                '$' => match chars.peek() {
                    Some(&'{') => self.output += "$$",
                    _ => self.output += "$",
                },
                '\n' => match multiline {
                    true => {
                        self.output += "\n";
                        self.indent();
                    }
                    false => self.output += "\\n",
                },
                '\t' => match multiline {
                    true => self.output += "\t",
                    false => self.output += "\\t",
                },
                c if c.is_control() => {
                    use std::fmt::Write;
                    write!(self.output, "\\x{:02x}", c as u32).unwrap();
                }
                c => self.output.push(c),
            }
        }

        match multiline {
            true => {
                self.output += "\n";
                self.indent_depth -= 1;
                self.indent();
                self.output += "''"
            }
            false => self.output += "\"",
        }

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.output += "[\n";

        for byte in v.iter() {
            self.output += "  ";
            self.serialize_u8(*byte)?;
            self.output += "\n";
        }

        self.output += "]";
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.output += "NULL";
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let lower: String = variant
            .chars()
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 && c.is_uppercase() {
                    c.to_lowercase()
                        .next()
                        .expect("This should be an iterable of one character.")
                } else {
                    c
                }
            })
            .collect();

        self.output += "{ ";
        self.output += &lower;
        self.output += " = ";
        value.serialize(&mut *self)?;
        self.output += "; }";
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.output += "[";
        self.indent_depth += 1;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.output += "{ ";
        variant.serialize(&mut *self)?;
        self.output += " = [";
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.output += "{";
        self.indent_depth += 1;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.output += "{ ";
        variant.serialize(&mut *self)?;
        self.output += " = {";
        Ok(self)
    }
}
