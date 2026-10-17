use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use time::OffsetDateTime;

pub(crate) fn data_dir() -> PathBuf {
    if cfg!(test) {
        return std::env::temp_dir().join("clavirio-test-data");
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".clavirio")
}

/// Loads a file that is about to be rewritten. If its content can't be parsed
/// or isn't valid UTF-8, it is first renamed to `<name>.<timestamp>.bak` so it
/// isn't lost. Returns `None` if the file can't be read or that rename fails,
/// meaning the file must not be rewritten.
pub(crate) fn load_for_rewrite<T: Default>(
    path: &Path,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Option<T> {
    let loaded = match fs::read(path) {
        Ok(bytes) => std::str::from_utf8(&bytes).ok().and_then(parse),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Some(T::default()),
        Err(_) => return None,
    };
    if loaded.is_none() {
        let stamp = OffsetDateTime::now_utc().unix_timestamp();
        let name = path.file_name()?.to_string_lossy();
        fs::rename(path, path.with_file_name(format!("{name}.{stamp}.bak"))).ok()?;
    }
    Some(loaded.unwrap_or_default())
}

/// Replaces `path` with `content` so that a crash or power cut leaves either the
/// old file or the new one, never a partial file or none. The content is written
/// to a temporary file in the same folder, flushed to disk, then renamed over
/// `path` in one step, on Windows too. Creates the folder if it is missing.
pub(crate) fn write_atomically(path: &Path, content: &str) -> io::Result<()> {
    let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no file name"));
    };
    fs::create_dir_all(dir)?;
    // A unique temporary name, so two saves at the same time can't mix their content
    static NEXT_TMP: AtomicU32 = AtomicU32::new(0);
    let mut tmp_name = name.to_os_string();
    tmp_name.push(format!(
        ".{}.{}.tmp",
        std::process::id(),
        NEXT_TMP.fetch_add(1, Ordering::Relaxed)
    ));
    let tmp = path.with_file_name(tmp_name);
    if let Err(e) = write_synced(&tmp, content).and_then(|()| fs::rename(&tmp, path)) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    sync_dir(dir);
    Ok(())
}

fn write_synced(path: &Path, content: &str) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()
}

/// Flushes the folder so the rename itself survives a power cut. Errors are
/// ignored: some file systems can't flush a folder, and the new file is in place
/// either way.
#[cfg(unix)]
fn sync_dir(dir: &Path) {
    if let Ok(handle) = fs::File::open(dir) {
        let _ = handle.sync_all();
    }
}

#[cfg(not(unix))]
fn sync_dir(_dir: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unparseable_file_kept_before_rewrite() {
        let dir = test_dir("backup");
        let path = dir.join("count.txt");
        let broken = "not a number";

        let missing = load_for_rewrite(&path, parse_number);
        fs::write(&path, broken).unwrap();
        let unparseable = load_for_rewrite(&path, parse_number);
        let files_left = file_contents(&dir);
        let _ = fs::remove_dir_all(&dir);

        assert!(missing.is_some());
        assert!(unparseable.is_some());
        assert_eq!(files_left, [broken.as_bytes()]);
    }

    #[test]
    fn non_utf8_file_kept_before_rewrite() {
        let dir = test_dir("backup-non-utf8");
        let path = dir.join("count.txt");
        let broken = [0xff, 0xfe, b'1', 0]; // "1" saved as UTF-16
        fs::write(&path, broken).unwrap();

        let loaded = load_for_rewrite(&path, parse_number);
        let files_left = file_contents(&dir);
        let _ = fs::remove_dir_all(&dir);

        assert!(loaded.is_some());
        assert_eq!(files_left, [broken]);
    }

    #[test]
    fn unreadable_file_left_alone() {
        let dir = test_dir("unreadable");
        let path = dir.join("count.txt");
        fs::create_dir(&path).unwrap(); // reading a folder fails, but not with "not found"

        let loaded = load_for_rewrite(&path, parse_number);
        let still_there = path.is_dir();
        let _ = fs::remove_dir_all(&dir);

        assert!(loaded.is_none());
        assert!(still_there);
    }

    #[test]
    fn write_atomically_replaces_file() {
        let dir = test_dir("atomic");
        let path = dir.join("settings.toml");

        write_atomically(&path, "old").unwrap();
        write_atomically(&path, "new").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let files_left = fs::read_dir(&dir).unwrap().count();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(content, "new");
        assert_eq!(files_left, 1);
    }

    #[test]
    fn write_atomically_creates_folder() {
        let dir = test_dir("atomic-folder");
        let path = dir.join("new").join("settings.toml");

        write_atomically(&path, "new").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(content, "new");
    }

    #[test]
    fn write_atomically_cleans_up_after_failure() {
        let dir = test_dir("atomic-failure");
        let path = dir.join("taken");
        fs::create_dir(&path).unwrap(); // a folder in the way, so the rename fails

        let result = write_atomically(&path, "new");
        let files_left = fs::read_dir(&dir).unwrap().count();
        let _ = fs::remove_dir_all(&dir);

        assert!(result.is_err());
        assert_eq!(files_left, 1);
    }

    fn parse_number(text: &str) -> Option<u32> {
        text.parse().ok()
    }

    fn file_contents(dir: &Path) -> Vec<Vec<u8>> {
        fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| fs::read(entry.ok()?.path()).ok())
            .collect()
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("clavirio-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
