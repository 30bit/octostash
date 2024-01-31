use serde::{
    de::{self, Error as _, IgnoredAny, MapAccess},
    Deserialize, Deserializer,
};

struct FindFilenames;

impl<'de> de::Visitor<'de> for FindFilenames {
    type Value = Filenames;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut output: Option<usize> = None;
        while let Some((key, _)) = map.next_entry::<&str, IgnoredAny>()? {
            if let Some(filename) = key.strip_prefix('_').and_then(|s| s.parse::<usize>().ok()) {
                if !output.map(|current| filename < current).unwrap_or_default() {
                    output = Some(filename);
                }
            }
        }
        Ok(Filenames { max_index: output })
    }
}

struct Filenames {
    max_index: Option<usize>,
}

impl<'de> Deserialize<'de> for Filenames {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FindFilenames)
    }
}

struct FindFilesLen;

impl<'de> de::Visitor<'de> for FindFilesLen {
    type Value = FilesLen;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map with `files` key")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<&str>()? {
            if key == "files" {
                let value = map.next_value()?;
                while map
                    .next_entry::<de::IgnoredAny, de::IgnoredAny>()?
                    .is_some()
                {}
                return Ok(FilesLen(value));
            } else {
                map.next_value::<de::IgnoredAny>()?;
            }
        }
        Err(A::Error::custom("no `files`"))
    }
}

pub struct FilesLen(Filenames);

impl<'de> Deserialize<'de> for FilesLen {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FindFilesLen)
    }
}

impl From<FilesLen> for usize {
    fn from(files: FilesLen) -> Self {
        files.0.max_index.unwrap_or_default() + 1
    }
}
