use std::array;

use serde::{
    de::{self, Error as _, SeqAccess},
    Deserialize, Deserializer,
};

struct FindId;

impl<'de> de::Visitor<'de> for FindId {
    type Value = IdRef<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map containing an `id` field")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<&str>()? {
            if key == "id" {
                let value = map.next_value()?;
                while map
                    .next_entry::<de::IgnoredAny, de::IgnoredAny>()?
                    .is_some()
                {}
                return Ok(IdRef(value));
            } else {
                map.next_value::<de::IgnoredAny>()?;
            }
        }
        Err(A::Error::custom("no `id`"))
    }
}

pub struct IdRef<'de>(&'de str);

impl<'de> Deserialize<'de> for IdRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FindId)
    }
}

pub struct Id(String);

impl<'de> From<IdRef<'de>> for Id {
    fn from(id: IdRef<'de>) -> Self {
        Self(id.0.into())
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FindId).map(Into::into)
    }
}

impl From<Id> for String {
    fn from(id: Id) -> Self {
        id.0
    }
}

pub struct CollectIdArray<const N: usize>;

impl<'de, const N: usize> de::Visitor<'de> for CollectIdArray<N> {
    type Value = IdArray<N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut buf = array::from_fn(|_| String::new());
        let mut index = 0;
        while let Some(Id(id)) = seq.next_element()? {
            *buf.get_mut(index)
                .ok_or_else(|| A::Error::custom("array length is insufficient"))? = id;
            index += 1;
        }
        Ok(IdArray { buf, len: index })
    }
}

pub struct IdArray<const N: usize> {
    buf: [String; N],
    len: usize,
}

impl<'de, const N: usize> Deserialize<'de> for IdArray<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(CollectIdArray)
    }
}

impl<const N: usize> From<IdArray<N>> for array::IntoIter<String, N> {
    fn from(ids: IdArray<N>) -> Self {
        let mut iter = ids.buf.into_iter();
        for _ in 0..N - ids.len {
            unsafe { iter.next_back().unwrap_unchecked() };
        }
        iter
    }
}
