use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess};
use serde::ser::{Serialize, Serializer};

use core::marker::PhantomData;
use core::fmt;

use crate::Array;

impl<T: Serialize, const S: usize> Serialize for Array<T, S> {
    #[inline]
    fn serialize<SER: Serializer>(&self, ser: SER) -> Result<SER::Ok, SER::Error> {
        ser.collect_seq(self.as_slice())
    }
}

impl<'de, T: Deserialize<'de>, const S: usize> Deserialize<'de> for Array<T, S> {
    fn deserialize<D: Deserializer<'de>>(des: D) -> Result<Self, D::Error> {
        struct ArrayVisitor<T, const S: usize>(PhantomData<[T; S]>);

        impl<'de, T: Deserialize<'de>, const S: usize> Visitor<'de> for ArrayVisitor<T, S> {
            type Value = Array<T, S>;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a capped sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut result = Self::Value::new();

                while let Some(value) = seq.next_element()? {
                    if result.push(value).is_some() {
                        return Err(serde::de::Error::custom(format_args!("Capacity({}) overflow", S)));
                    }
                }

                Ok(result)
            }
        }

        des.deserialize_seq(ArrayVisitor(PhantomData))
    }
}
