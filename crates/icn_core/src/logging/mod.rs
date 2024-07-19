// src/logging.rs
pub fn log_info(message: &str) {
    println!("[INFO] {}", message);
}

pub fn log_warn(message: &str) {
    println!("[WARN] {}", message);
}

pub fn log_error(message: &str) {
    eprintln!("[ERROR] {}", message);
}

pub fn log_debug(message: &str) {
    println!("[DEBUG] {}", message);
}
