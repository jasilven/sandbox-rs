use anyhow::Result;
use std::fs::{self, DirEntry};
use std::path::Path;

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry) -> Result<()>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

fn entry_handler(entry: &DirEntry) -> Result<()> {
    let meta = entry.metadata()?;
    let now = std::time::SystemTime::now();

    if meta.is_file() {
        let modified = meta.modified()?;
        let duration = now.duration_since(modified)?;
        let day = 60 * 60 * 24;
        println!("{:?} {:?}", entry.file_name(), duration.as_secs() / day);
    }
    Ok(())
}
fn main() -> Result<()> {
    visit_dirs(Path::new("."), &entry_handler)?;

    Ok(())
}
