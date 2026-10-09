use crate::{
    config::allowed_usage,
    protocol,
    runtime::{Issue, Output, Transport},
};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::{Path, PathBuf},
};

fn device_issue(error: io::Error) -> Issue {
    let code = match error.raw_os_error() {
        Some(libc::EACCES | libc::EPERM) => "PERMISSION_REQUIRED",
        Some(libc::EBUSY | libc::EAGAIN) => "DEVICE_BUSY",
        _ => "DEVICE_MISSING",
    };
    Issue::new(code, "receiver", error)
}

// 读取 HID 描述符的短项，检查厂商 usage page 和 Report ID，不依赖枚举序号。
pub fn vendor_descriptor(bytes: &[u8]) -> bool {
    let (mut offset, mut page, mut report, mut size, mut count) = (0, 0, 0, 0, 0);
    let (mut input, mut output) = (false, false);
    let mut stack = Vec::new();
    while offset < bytes.len() {
        let prefix = bytes[offset];
        offset += 1;
        if prefix == 0xfe {
            if offset + 2 > bytes.len() {
                return false;
            }
            let n = bytes[offset] as usize;
            offset += 2 + n;
            if offset > bytes.len() {
                return false;
            }
            continue;
        }
        let n = match prefix & 3 {
            3 => 4,
            x => x as usize,
        };
        if offset + n > bytes.len() {
            return false;
        }
        let value = bytes[offset..offset + n]
            .iter()
            .enumerate()
            .fold(0u32, |v, (i, b)| v | (*b as u32) << (8 * i));
        offset += n;
        match prefix & 0xfc {
            0x04 => page = value,
            0x84 => report = value,
            0x74 => size = value,
            0x94 => count = value,
            0xa4 => stack.push((page, report, size, count)),
            0xb4 => match stack.pop() {
                Some(s) => (page, report, size, count) = s,
                None => return false,
            },
            0x80 if page == 0xfffc && report == 0x55 && size == 8 && count == 63 => input = true,
            0x90 if page == 0xfffc && report == 0x55 && size == 8 && count == 63 => output = true,
            _ => (),
        }
    }
    input && output
}

#[derive(Default)]
pub struct Hidraw {
    file: Option<File>,
    path: Option<PathBuf>,
}
impl Transport for Hidraw {
    fn connect(&mut self) -> Result<(), Issue> {
        self.close();
        let mut paths = Vec::new();
        let entries = match fs::read_dir("/sys/class/hidraw") {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(Issue::new("DEVICE_MISSING", "receiver", "no HID receivers"))
            }
            Err(e) => return Err(device_issue(e)),
        };
        for entry in entries {
            let entry = entry.map_err(device_issue)?;
            let device = entry.path().join("device");
            let uevent = fs::read_to_string(device.join("uevent")).map_err(device_issue)?;
            let matches = uevent
                .lines()
                .find_map(|l| l.strip_prefix("HID_ID="))
                .is_some_and(|id| {
                    let parts: Vec<_> = id.split(':').collect();
                    parts.len() == 3
                        && u32::from_str_radix(parts[1], 16) == Ok(protocol::VID.into())
                        && u32::from_str_radix(parts[2], 16) == Ok(protocol::PID.into())
                });
            if matches
                && vendor_descriptor(
                    &fs::read(device.join("report_descriptor")).map_err(device_issue)?,
                )
            {
                paths.push(Path::new("/dev").join(entry.file_name()));
            }
        }
        if paths.is_empty() {
            return Err(Issue::new(
                "DEVICE_MISSING",
                "receiver",
                "AU05 vendor interface not found",
            ));
        }
        if paths.len() != 1 {
            return Err(Issue::new(
                "DEVICE_MULTIPLE",
                "receiver",
                "disconnect extra receivers and retry",
            ));
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(&paths[0])
            .map_err(device_issue)?;
        // hidraw 没有强制独占接口；此锁防止遵循 flock 的客户端并发接管。
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(device_issue(io::Error::last_os_error()));
        }
        self.path = paths.pop();
        self.file = Some(file);
        Ok(())
    }
    fn read(&mut self) -> Result<Option<Vec<u8>>, Issue> {
        if self.path.as_ref().is_none_or(|p| !p.exists()) {
            return Err(Issue::new("DEVICE_MISSING", "receiver", "receiver removed"));
        }
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(self.path.as_ref().unwrap().as_os_str().as_bytes())
            .map_err(|e| Issue::new("DEVICE_MISSING", "receiver", e))?;
        if unsafe { libc::access(path.as_ptr(), libc::R_OK | libc::W_OK) } != 0 {
            return Err(device_issue(io::Error::last_os_error()));
        }
        let mut data = [0u8; 65];
        match self.file.as_mut().unwrap().read(&mut data) {
            Ok(0) => Err(Issue::new("DEVICE_MISSING", "receiver", "receiver closed")),
            Ok(n) => Ok(Some(data[..n].to_vec())),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(device_issue(e)),
        }
    }
    fn send(&mut self, command: &[u8]) -> Result<(), Issue> {
        let report =
            protocol::encode(command).map_err(|e| Issue::new("HANDOVER_FAILED", "receiver", e))?;
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| Issue::new("DEVICE_MISSING", "receiver", "receiver closed"))?;
        match file.write(&report) {
            Ok(64) => Ok(()),
            Ok(_) => Err(Issue::new("HANDOVER_FAILED", "receiver", "short HID write")),
            Err(e) => Err(device_issue(e)),
        }
    }
    fn close(&mut self) {
        self.file = None;
        self.path = None;
    }
}

pub fn linux_key(usage: u16) -> Option<u16> {
    const LETTERS: [u16; 26] = [
        30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20, 22, 47, 17,
        45, 21, 44,
    ];
    Some(match usage {
        4..=29 => LETTERS[(usage - 4) as usize],
        30..=38 => usage - 28,
        39 => 11,
        40 => 28,
        41 => 1,
        42 => 14,
        43 => 15,
        44 => 57,
        45 => 12,
        46 => 13,
        47 => 26,
        48 => 27,
        49 => 43,
        51 => 39,
        52 => 40,
        53 => 41,
        54 => 51,
        55 => 52,
        56 => 53,
        58..=67 => usage + 1,
        68 => 87,
        69 => 88,
        73 => 110,
        74 => 102,
        75 => 104,
        76 => 111,
        77 => 107,
        78 => 109,
        79 => 106,
        80 => 105,
        81 => 108,
        82 => 103,
        224 => 29,
        225 => 42,
        226 => 56,
        227 => 125,
        _ => return None,
    })
}

#[derive(Default)]
pub struct UInput {
    file: Option<File>,
    ready_at: Option<std::time::Instant>,
}
impl UInput {
    fn create() -> io::Result<File> {
        let file = OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open("/dev/uinput")?;
        let ioctl = |request: libc::c_ulong, value: i32| -> io::Result<()> {
            if unsafe { libc::ioctl(file.as_raw_fd(), request, value) } < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        };
        ioctl(0x40045564, 1)?; // UI_SET_EVBIT(EV_KEY)
        for usage in (4..=82).filter(|u| allowed_usage(*u)).chain(224..=227) {
            ioctl(0x40045565, linux_key(usage).unwrap().into())?; // UI_SET_KEYBIT
        }
        // Linux uinput_setup 的稳定 ABI；不用普通键盘事件节点，不监听文本。
        let mut descriptor = [0u8; 92];
        let name = b"clanzi virtual keyboard";
        descriptor[8..8 + name.len()].copy_from_slice(name);
        descriptor[..2].copy_from_slice(&6u16.to_ne_bytes()); // BUS_VIRTUAL
        if unsafe { libc::ioctl(file.as_raw_fd(), 0x405c5503, descriptor.as_ptr()) } < 0 {
            return Err(io::Error::last_os_error());
        }
        if unsafe { libc::ioctl(file.as_raw_fd(), 0x5501) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(file)
    }
}
impl Output for UInput {
    fn ensure(&mut self) -> Result<(), Issue> {
        // 已打开的 fd 在撤销 ACL 后仍然可写，所以也复查当前路径权限。
        if unsafe { libc::access(c"/dev/uinput".as_ptr(), libc::W_OK) } != 0 {
            return Err(Issue::new(
                "PERMISSION_REQUIRED",
                "output",
                io::Error::last_os_error(),
            ));
        }
        if self.file.is_none() {
            self.file =
                Some(Self::create().map_err(|e| Issue::new("PERMISSION_REQUIRED", "output", e))?);
            // 给桌面输入栈注册新设备的时间；此等待不阻塞查询及心跳线程。
            self.ready_at = Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
        }
        Ok(())
    }
    fn ready(&self) -> bool {
        self.file.is_some()
            && self
                .ready_at
                .is_some_and(|at| std::time::Instant::now() >= at)
    }
    fn key(&mut self, usage: u16, down: bool) -> Result<(), Issue> {
        let code = linux_key(usage)
            .ok_or_else(|| Issue::new("OUTPUT_FAILED", "output", "unsupported usage"))?;
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| Issue::new("OUTPUT_FAILED", "output", "virtual keyboard unavailable"))?;
        let event = |kind: u16, code: u16, value: i32| {
            let mut data = vec![0u8; std::mem::size_of::<libc::timeval>()];
            data.extend(kind.to_ne_bytes());
            data.extend(code.to_ne_bytes());
            data.extend(value.to_ne_bytes());
            data
        };
        let bytes = [event(1, code, i32::from(down)), event(0, 0, 0)].concat();
        // 每次写入包含 SYN_REPORT，失败仍由调度器保留松开债务。
        file.write_all(&bytes)
            .map_err(|e| Issue::new("OUTPUT_FAILED", "output", e))
    }
    fn close(&mut self) {
        self.ready_at = None;
        if let Some(file) = self.file.take() {
            unsafe {
                libc::ioctl(file.as_raw_fd(), 0x5502);
            }
        }
    }
}
impl Drop for UInput {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Default)]
pub struct DemoTransport {
    replies: std::collections::VecDeque<Vec<u8>>,
}
impl Transport for DemoTransport {
    fn connect(&mut self) -> Result<(), Issue> {
        Ok(())
    }
    fn read(&mut self) -> Result<Option<Vec<u8>>, Issue> {
        Ok(self.replies.pop_front())
    }
    fn send(&mut self, command: &[u8]) -> Result<(), Issue> {
        protocol::encode(command).map_err(|e| Issue::new("HANDOVER_FAILED", "device", e))?;
        if command == protocol::QUERY {
            self.replies
                .push_back(protocol::encode_frame(&[6, 3, 10, 17, 1]).to_vec());
        }
        Ok(())
    }
    fn close(&mut self) {
        self.replies.clear();
    }
}
pub struct DemoOutput;
impl Output for DemoOutput {
    fn ensure(&mut self) -> Result<(), Issue> {
        Ok(())
    }
    fn key(&mut self, _: u16, _: bool) -> Result<(), Issue> {
        Ok(())
    }
    fn close(&mut self) {}
}
