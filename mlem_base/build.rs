use std::time::{ SystemTime, UNIX_EPOCH };

fn main() {
    write_build_id();
}

fn write_build_id() {
    let time = SystemTime::now();
    let since = time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let id = format!("{:x}", since.as_secs());
    println!("cargo:rustc-env=BUILD_ID={}", id);
}