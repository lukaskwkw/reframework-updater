use log::debug;
use rand::{self, thread_rng, Rng};
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

struct Chunks<R> {
    read: R,
    size: usize,
    hint: (usize, Option<usize>),
}

impl<R> Chunks<R> {
    pub fn new(read: R, size: usize) -> Self {
        Self {
            read,
            size,
            hint: (0, None),
        }
    }

    pub fn from_seek(mut read: R, size: usize) -> io::Result<Self>
    where
        R: Seek,
    {
        let old_pos = read.seek(SeekFrom::Current(0))?;
        let len = read.seek(SeekFrom::End(0))?;

        let rest = (len - old_pos) as usize; // len is always >= old_pos but they are u64
        if rest != 0 {
            read.seek(SeekFrom::Start(old_pos))?;
        }

        let min = rest / size + if rest % size != 0 { 1 } else { 0 };
        Ok(Self {
            read,
            size,
            hint: (min, None), // this could be wrong I'm unsure
        })
    }

    // This could be useful if you want to try to recover from an error
    pub fn into_inner(self) -> R {
        self.read
    }
}

const BUFFER_SIZE: usize = 0xFF;

impl<R> Iterator for Chunks<R>
where
    R: Read,
{
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.size);
        match self
            .read
            .by_ref()
            .take(chunk.capacity() as u64)
            .read_to_end(&mut chunk)
        {
            Ok(n) => {
                if n != 0 {
                    Some(Ok(chunk))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.hint
    }
}

trait ReadPlus: Read {
    fn chunks(self, size: usize) -> Chunks<Self>
    where
        Self: Sized,
    {
        Chunks::new(self, size)
    }
}

impl<T: ?Sized> ReadPlus for T where T: Read {}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

pub fn find_string_in_binary_file(file: impl AsRef<Path>, text: &str) -> io::Result<bool> {
    let mut file = std::fs::File::open(file)?;
    let mut is_success = perform_find_in_chunks(&file, text, BUFFER_SIZE)?;
    if !is_success {
        let mut rng = thread_rng();
        let random_plus: u32 = rng.gen_range(10..256);

        let new_buff_size = BUFFER_SIZE + random_plus as usize;
        debug!(
            "\nNot found in chunks in {:?} Searching one more time with new buffer size {}. \n",
            file, new_buff_size
        );
        file.rewind()?;

        is_success = perform_find_in_chunks(&file, text, BUFFER_SIZE + random_plus as usize)?;
    }

    Ok(is_success)
}

fn perform_find_in_chunks(file: &File, text: &str, chunk_size: usize) -> io::Result<bool> {
    let iter = Chunks::from_seek(file, chunk_size)?;
    debug!("size hint {:?}", iter.size_hint());
    let chunks = iter.collect::<Result<Vec<_>, _>>()?;
    debug!("len {:?}, capacity {:?}", chunks.len(), chunks.capacity());

    let mut index = 0;
    let mut able_to_found: bool = false;
    while index < chunks.len() {
        let found = find_subsequence(&chunks[index], text.as_bytes());
        index += 1;

        match found {
            Some(pos) => {
                able_to_found = true;
                debug!("found at {} at {} chunk", pos, index);
                break;
            }
            None => continue,
        }
    }
    Ok(able_to_found)
}

#[cfg(test)]
mod tests {
    use crate::utils::{binSearch::find_subsequence, init_logger::init_logger};

    fn init() {
        init_logger("debug");
    }

    // #[test]
    // fn testBin() {
    //     init();
    //     let is_success = find_string_in_binary_file("tmp/dinput8_re7.dll", "RE7_TDB").unwrap();
    //     assert!(is_success);
    // }

    #[test]
    fn find_subsequence_test() {
        assert_eq!(find_subsequence(b"qwertyuiop", b"tyu"), Some(4));
        assert_eq!(find_subsequence(b"qwertyuiop", b"asd"), None);
    }
}
