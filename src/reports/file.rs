use std::fs::File;
use std::path::Path;
use std::sync::mpsc::Receiver;

pub struct FileReporter {
    pub file: File,
    recv: Receiver<Entry>,
}

pub struct Entry {
    timestamp: u32,
}

impl FileReporter {
    pub fn new(location: String, rv: Receiver<Entry>) -> Result<FileReporter, std::io::Error> {
        let path = Path::new(&location);
        let file = File::open(path)?;

        Ok(FileReporter {
            file: file,
            recv: rv,
        })
    }

    pub fn listen(&self) {
        loop {
            match self.recv.recv() {
                Ok(entry) => (
                    // TODO write to file
                    // self.file.wr
                ),
                // TODO Consider some kind of logging to know why the channel was closed (other than normal hang up)
                Err(err) => (),
            }
        }
    }
}
