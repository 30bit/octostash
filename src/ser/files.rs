use serde::{ser::SerializeMap as _, Serialize, Serializer};

struct FileContent<'a>(&'a str);

impl<'a> Serialize for FileContent<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("content", self.0)?;
        map.end()
    }
}

struct FilesContentChunks<'a>(&'a str);

const FILE_MAX_CHARS: usize = 250000;

impl<'a> Iterator for FilesContentChunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else if let Some((file_end, _)) = self.0.char_indices().nth(FILE_MAX_CHARS) {
            let (file, remainder) = self.0.split_at(file_end);
            self.0 = remainder;
            Some(file)
        } else {
            let file = self.0;
            self.0 = "";
            Some(file)
        }
    }
}

struct FilenameBuffer([u8; 21]);

impl FilenameBuffer {
    fn new() -> Self {
        let mut bytes = [0; 21];
        bytes[0] = '_' as u8;
        Self(bytes)
    }

    fn fmt(&mut self, i: usize) -> &str {
        let mut itoa_buf = itoa::Buffer::new();
        let bytes = itoa_buf.format(i).as_bytes();
        let end = bytes.len() + 1;
        unsafe { self.0.get_unchecked_mut(1..end) }.copy_from_slice(bytes);
        unsafe { std::str::from_utf8_unchecked(&self.0[..end]) }
    }
}

struct FilesContent<'a> {
    files_content: &'a str,
    current_len: usize,
}

impl<'a> Serialize for FilesContent<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let new_count = FilesContentChunks(self.files_content).count();
        let mut map = serializer.serialize_map(Some(new_count))?;
        let mut filename_buf = FilenameBuffer::new();
        for (index, file_content) in FilesContentChunks(self.files_content).enumerate() {
            map.serialize_entry(filename_buf.fmt(index), &FileContent(file_content))?;
        }
        for index in new_count..self.current_len {
            map.serialize_entry(filename_buf.fmt(index), &())?;
        }
        map.end()
    }
}

pub struct Files<'a>(FilesContent<'a>);

impl<'a> Serialize for Files<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("files", &self.0)?;
        map.end()
    }
}

impl<'a> Files<'a> {
    pub fn new(files_content: &'a str, current_len: usize) -> Self {
        Self(FilesContent {
            files_content,
            current_len,
        })
    }
}
