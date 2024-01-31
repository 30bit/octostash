use serde::{
    de::{self, Error as _, IgnoredAny, MapAccess},
    Deserialize, Deserializer,
};

struct FindFileContent;

impl<'de> de::Visitor<'de> for FindFileContent {
    type Value = FileContent<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map with `content` key")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<&str>()? {
            if key == "content" {
                let value = map.next_value()?;
                while map
                    .next_entry::<de::IgnoredAny, de::IgnoredAny>()?
                    .is_some()
                {}
                return Ok(FileContent(value));
            } else {
                map.next_value::<de::IgnoredAny>()?;
            }
        }
        Err(A::Error::custom("no `content`"))
    }
}

struct FileContent<'de>(&'de str);

impl<'de> Deserialize<'de> for FileContent<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FindFileContent)
    }
}

struct CollectFilesContent;

impl<'de> de::Visitor<'de> for CollectFilesContent {
    type Value = FilesContent;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut output = String::new();
        while let Some((_, FileContent(content))) = map.next_entry::<IgnoredAny, FileContent>()? {
            output.push_str(content);
        }
        Ok(FilesContent(output))
    }
}

struct FilesContent(String);

impl<'de> Deserialize<'de> for FilesContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(CollectFilesContent)
    }
}

struct FindFiles;

impl<'de> de::Visitor<'de> for FindFiles {
    type Value = Files;

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
                return Ok(Files {
                    files_content: value,
                });
            } else {
                map.next_value::<de::IgnoredAny>()?;
            }
        }
        Err(A::Error::custom("no `files`"))
    }
}

pub struct Files {
    files_content: FilesContent,
}

impl<'de> Deserialize<'de> for Files {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(FindFiles)
    }
}

impl From<Files> for String {
    fn from(x: Files) -> Self {
        x.files_content.0
    }
}
