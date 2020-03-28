pub mod file {
    use std::fs::File;
    use std::fs::OpenOptions;
    use std::path::Path;

    pub trait Append {
        fn create_append<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
            OpenOptions::new()
                .truncate(false)
                .append(true)
                .create(true)
                .open(path.as_ref())
        }
    }
    impl Append for std::fs::File {}
}
