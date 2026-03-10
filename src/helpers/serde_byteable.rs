use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{DeserializeOwned, DeserializeSeed, SeqAccess},
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
};
use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};

use crate::errors::{DecodeError, EncodeError};

pub async fn encode_serde<W, T>(v: &T, writer: &mut W) -> Result<(), EncodeError>
where
    W: AsyncWrite + Unpin + Send,
    T: Serialize,
{
    let mut ser = ByteableSerializer(Vec::new());
    v.serialize(&mut ser)?;
    writer.write_u64(ser.0.len() as u64).await?;
    writer.write(&ser.0).await?;
    Ok(())
}

pub async fn decode_serde<'de, R, T>(reader: &mut R) -> Result<T, DecodeError>
where
    R: AsyncRead + Unpin + Send,
    T: DeserializeOwned,
{
    let len = reader.read_u64().await?;
    let mut bytes: Vec<u8> = Vec::with_capacity(len as usize);
    unsafe {
        bytes.set_len(len as usize);
    }

    reader.read_exact(&mut bytes).await?;
    let deserialized = {
        let mut de = ByteableDeserializer(&bytes);
        T::deserialize(&mut de)?
    };

    Ok(deserialized)
}

struct ByteableSerializer(Vec<u8>);
struct ByteableDeserializer<'de>(&'de [u8]);

impl<'a> SerializeSeq for &'a mut ByteableSerializer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeTuple for &'a mut ByteableSerializer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeTupleStruct for &'a mut ByteableSerializer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeTupleVariant for &'a mut ByteableSerializer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeMap for &'a mut ByteableSerializer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeStruct for &'a mut ByteableSerializer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeStructVariant for &'a mut ByteableSerializer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> Serializer for &'a mut ByteableSerializer {
    type Ok = ();

    type Error = EncodeError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.0.push(v as u8);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.0.push(v as u8);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.0.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.0.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.0.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.0.push(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.0.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.0.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.0.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.0.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.0.extend_from_slice(&v.to_be_bytes());
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.0.push(v as u8);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.0.extend((v.len() as u16).to_be_bytes());
        self.0.extend_from_slice(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.0.extend((v.len() as u16).to_be_bytes());
        self.0.extend_from_slice(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.0.push(0u8);
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.push(1u8);
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        variant_index.serialize(self)
    }

    fn serialize_newtype_struct<T>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // variant_index.serialize(*x)?;
        // value.serialize(self)
        unimplemented!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        match len {
            Some(len) => {
                if len > u16::MAX as usize {
                    return Err(EncodeError::InvalidData);
                }
                self.0.extend((len as u16).to_be_bytes());
            }
            None => return Err(EncodeError::InvalidData),
        }
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // variant_index.serialize(self)?;
        // Ok(self)
        unimplemented!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        match len {
            Some(len) => {
                if len > u16::MAX as usize {
                    return Err(EncodeError::InvalidData);
                }
                self.0.extend((len as u16).to_be_bytes());
            }
            None => return Err(EncodeError::InvalidData),
        }
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        // variant_index.serialize(self)?;
        // Ok(())
        unimplemented!()
    }
}

impl<'de, 'a> Deserializer<'de> for &'a mut ByteableDeserializer<'de> {
    type Error = DecodeError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(1) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_bool(value[0] != 0)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(1) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_i8(value[0] as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(2) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_i16(i16::from_be_bytes(value.try_into().unwrap()))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(4) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_i32(i32::from_be_bytes(value.try_into().unwrap()))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(8) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_i64(i64::from_be_bytes(value.try_into().unwrap()))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(1) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_u8(value[0])
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(2) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_u16(u16::from_be_bytes(value.try_into().unwrap()))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(4) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_u32(u32::from_be_bytes(value.try_into().unwrap()))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(8) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_u64(u64::from_be_bytes(value.try_into().unwrap()))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(4) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_f32(f32::from_be_bytes(value.try_into().unwrap()))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(8) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_f64(f64::from_be_bytes(value.try_into().unwrap()))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(1) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        visitor.visit_char(value[0] as char)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((len, rem)) = self.0.split_at_checked(2) else {
            return Err(DecodeError::InvalidData);
        };
        let len = u16::from_be_bytes(len.try_into().unwrap());
        self.0 = rem;

        let Some((value, rem)) = self.0.split_at_checked(len as usize) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;

        visitor.visit_str(std::str::from_utf8(value).unwrap())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((len, rem)) = self.0.split_at_checked(2) else {
            return Err(DecodeError::InvalidData);
        };
        let len = u16::from_be_bytes(len.try_into().unwrap());
        self.0 = rem;

        let Some((value, rem)) = self.0.split_at_checked(len as usize) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;

        visitor.visit_string(std::str::from_utf8(value).unwrap().to_string())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((len, rem)) = self.0.split_at_checked(2) else {
            return Err(DecodeError::InvalidData);
        };
        let len = u16::from_be_bytes(len.try_into().unwrap());
        self.0 = rem;

        let Some((value, rem)) = self.0.split_at_checked(len as usize) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;

        visitor.visit_bytes(value)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((len, rem)) = self.0.split_at_checked(2) else {
            return Err(DecodeError::InvalidData);
        };
        let len = u16::from_be_bytes(len.try_into().unwrap());
        self.0 = rem;

        let Some((value, rem)) = self.0.split_at_checked(len as usize) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;

        visitor.visit_byte_buf(value.to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((value, rem)) = self.0.split_at_checked(1) else {
            return Err(DecodeError::InvalidData);
        };
        self.0 = rem;
        match value[0] {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(DecodeError::InvalidData),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let Some((len, rem)) = self.0.split_at_checked(2) else {
            return Err(DecodeError::InvalidData);
        };
        let len = u16::from_be_bytes(len.try_into().unwrap());
        self.0 = rem;

        visitor.visit_seq(CommaSeparated { de: self, len })
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let len = fields.len();
        if len > u16::MAX as usize {
            return Err(DecodeError::InvalidData);
        }
        let len = len as u16;
        visitor.visit_seq(CommaSeparated { de: self, len })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }
}

struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut ByteableDeserializer<'de>,
    len: u16,
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = DecodeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;

        seed.deserialize(&mut *self.de).map(Some)
    }
}
