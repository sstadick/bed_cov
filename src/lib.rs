pub mod bed_reader {
    use std::{
        fs::File,
        io::{self, prelude::*},
    };

    pub struct BufReader {
        reader: io::BufReader<File>,
    }

    impl BufReader {
        pub fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
            let file = File::open(path)?;
            let reader = io::BufReader::new(file);
            Ok(Self { reader })
        }

        pub fn read_line<'buf>(
            &mut self,
            buffer: &'buf mut String,
        ) -> io::Result<Option<(&'buf str, &'buf str, &'buf str)>> {
            buffer.clear();
            match self.reader.read_line(buffer) {
                Ok(0) => Ok(None),
                Ok(_) => {
                    let mut iter = buffer[..buffer.len() - 1].split('\t');
                    let chr = iter.next().unwrap();
                    let start = iter.next().unwrap();
                    let stop = iter.next().unwrap();
                    Ok(Some((chr, start, stop)))
                }
                Err(e) => Err(e),
            }
        }
    }
}
