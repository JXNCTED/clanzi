pub const VID: u16 = 0xfff1;
pub const PID: u16 = 0x00dd;
pub const REPORT_ID: u8 = 0x55;
pub const QUERY: &[u8] = &[0x06, 0x03, 0x0a, 0x01];
pub const TAKEOVER: &[u8] = &[0x01, 0x01, 0x10, 0x00, 0x03];
pub const HEARTBEAT: &[u8] = &[0x06, 0x01, 0x23, 0x00, 0x01];
pub const HAND_BACK: &[u8] = &[0x01, 0x01, 0x10, 0x00, 0x00];
const KEY: [u32; 4] = [0xcaa5baca, 0xbc2a8a6d, 0xca5a9eba, 0x9bb88bca];
const DELTA: u32 = 0x9e3779b9;

pub fn tea(block: [u8; 8], decrypt: bool) -> [u8; 8] {
    let mut a = u32::from_le_bytes(block[..4].try_into().unwrap());
    let mut b = u32::from_le_bytes(block[4..].try_into().unwrap());
    let mix = |v: u32, sum: u32, k0: u32, k1: u32| {
        (v << 4).wrapping_add(k0) ^ v.wrapping_add(sum) ^ (v >> 5).wrapping_add(k1)
    };
    let mut sum: u32 = if decrypt { 0xc6ef3720 } else { 0 };
    for _ in 0..32 {
        if decrypt {
            b = b.wrapping_sub(mix(a, sum, KEY[2], KEY[3]));
            a = a.wrapping_sub(mix(b, sum, KEY[0], KEY[1]));
            sum = sum.wrapping_sub(DELTA);
        } else {
            sum = sum.wrapping_add(DELTA);
            a = a.wrapping_add(mix(b, sum, KEY[0], KEY[1]));
            b = b.wrapping_add(mix(a, sum, KEY[2], KEY[3]));
        }
    }
    let mut out = [0; 8];
    out[..4].copy_from_slice(&a.to_le_bytes());
    out[4..].copy_from_slice(&b.to_le_bytes());
    out
}

pub fn encode(command: &[u8]) -> Result<[u8; 64], &'static str> {
    if ![QUERY, TAKEOVER, HEARTBEAT, HAND_BACK].contains(&command) {
        return Err("command not allowed");
    }
    Ok(encode_frame(command))
}

pub fn encode_frame(frame: &[u8]) -> [u8; 64] {
    assert!(frame.len() <= 64);
    let mut plain = [0; 64];
    plain[..frame.len()].copy_from_slice(frame);
    let mut encrypted = [0; 64];
    for (src, dst) in plain.chunks_exact(8).zip(encrypted.chunks_exact_mut(8)) {
        dst.copy_from_slice(&tea(src.try_into().unwrap(), false));
    }
    let mut report = [0; 64];
    report[0] = REPORT_ID;
    report[1..].copy_from_slice(&encrypted[..63]);
    report
}

pub fn decode(report: &[u8]) -> Result<[u8; 56], &'static str> {
    if report.len() != 64 || report[0] != REPORT_ID {
        return Err("invalid report length or ID");
    }
    let mut plain = [0; 56];
    for (src, dst) in report[1..57].chunks_exact(8).zip(plain.chunks_exact_mut(8)) {
        dst.copy_from_slice(&tea(src.try_into().unwrap(), true));
    }
    Ok(plain)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Control { index: u8, down: bool },
    Status(Option<bool>),
    Other,
}

pub fn parse(frame: &[u8]) -> Result<Event, &'static str> {
    if frame.len() < 5 {
        return Err("short frame");
    }
    if frame[0] & 0x1f == 0x0b && frame[1] == 0x10 {
        if frame[4] > 5 || frame[3] > 1 {
            return Err("invalid control edge");
        }
        return Ok(Event::Control {
            index: frame[4],
            down: frame[3] == 1,
        });
    }
    if frame[0] & 0x1f == 6 && frame[1..4] == [3, 10, 17] {
        return Ok(Event::Status(match frame[4] {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }));
    }
    Ok(Event::Other)
}
