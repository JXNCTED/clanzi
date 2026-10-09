use crate::config::{Keymap, LIMIT};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

pub fn config_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        });
    base.join("olanzi-cross-platform")
}

pub fn read_limited(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut data)?;
    if data.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file exceeds 64 KiB",
        ));
    }
    Ok(data)
}

pub fn timestamp() -> String {
    // localtime_r 不共享可变的静态缓冲。
    unsafe {
        let time = libc::time(std::ptr::null_mut());
        let mut local: libc::tm = std::mem::zeroed();
        libc::localtime_r(&time, &mut local);
        let mut out = [0u8; 32];
        let n = libc::strftime(
            out.as_mut_ptr().cast(),
            out.len(),
            c"%Y%m%d-%H%M%S".as_ptr(),
            &local,
        );
        String::from_utf8_lossy(&out[..n]).into_owned()
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing parent"))?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".clanzi-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        let directory = File::open(parent)?;
        directory.sync_all()?;
        fs::rename(&temp, path)?;
        // rename 是提交点；之后的目录同步错误不能谎报旧文件仍然有效。
        if let Err(error) = directory.sync_all() {
            eprintln!("CONFIG_DIRECTORY_SYNC: {error}");
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

pub struct Store {
    pub directory: PathBuf,
}
impl Store {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }
    pub fn load(&self) -> Result<Option<Keymap>, String> {
        match read_limited(&self.directory.join("keymap.json"), LIMIT) {
            Ok(data) => Keymap::parse(&data).map(Some),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn save(&self, map: &Keymap) -> Result<(), String> {
        let map = map.clone().validate()?;
        fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        let path = self.directory.join("keymap.json");
        if self.load().is_err() && path.exists() {
            let stamp = timestamp();
            for n in 0u64.. {
                let backup = self
                    .directory
                    .join(format!("keymap.invalid-{stamp}-{n}.json"));
                match OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&backup)
                {
                    Ok(mut target) => {
                        let result = File::open(&path)
                            .and_then(|mut source| io::copy(&mut source, &mut target))
                            .and_then(|_| target.sync_all());
                        if let Err(e) = result {
                            let _ = fs::remove_file(backup);
                            return Err(e.to_string());
                        }
                        File::open(&self.directory)
                            .and_then(|d| d.sync_all())
                            .map_err(|e| e.to_string())?;
                        break;
                    }
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
        atomic_write(&path, &map.bytes()).map_err(|e| e.to_string())
    }
    pub fn language(&self) -> String {
        #[derive(Deserialize)]
        struct Preferences {
            language: String,
        }
        read_limited(&self.directory.join("preferences.json"), 1024)
            .ok()
            .and_then(|b| serde_json::from_slice::<Preferences>(&b).ok())
            .map(|p| p.language)
            .filter(|s| s == "en" || s == "zh")
            .unwrap_or_else(|| {
                let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
                    .iter()
                    .find_map(|k| std::env::var(k).ok().filter(|s| !s.is_empty()))
                    .unwrap_or_default();
                if locale.starts_with("zh") { "zh" } else { "en" }.into()
            })
    }
    pub fn set_language(&self, language: &str) -> Result<(), String> {
        #[derive(Serialize)]
        struct Preferences<'a> {
            language: &'a str,
        }
        if !["en", "zh"].contains(&language) {
            return Err("invalid language".into());
        }
        atomic_write(
            &self.directory.join("preferences.json"),
            &serde_json::to_vec(&Preferences { language }).unwrap(),
        )
        .map_err(|e| e.to_string())
    }
}

pub fn private_directory(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}
