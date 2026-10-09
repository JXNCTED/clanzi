use clanzi_core::{
    config::{Keymap, LIMIT},
    storage::read_limited,
};
fn main() {
    let result = std::env::args_os()
        .nth(1)
        .ok_or("usage: clanzi-validate FILE".to_string())
        .and_then(|p| read_limited(std::path::Path::new(&p), LIMIT).map_err(|e| e.to_string()))
        .and_then(|b| Keymap::parse(&b));
    match result {
        Ok(map) => println!("{}", String::from_utf8(map.bytes()).unwrap()),
        Err(error) => {
            eprintln!("CONFIG_INVALID: {error}");
            std::process::exit(1);
        }
    }
}
