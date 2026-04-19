use std::fs;
use std::process::Command;
use chrono::Local;

fn main() {
    let now = Local::now();
    let date_str = now.format("%Y%m%d").to_string();
    
    // Пытаемся прочитать текущий счетчик билдов из временного файла
    let build_file = "build_counter.tmp";
    let mut count = 1;
    if let Ok(content) = fs::read_to_string(build_file) {
        if let Ok(saved_count) = content.trim().parse::<u32>() {
            count = saved_count + 1;
        }
    }
    let _ = fs::write(build_file, count.to_string());

    let version = format!("V{}_{}", date_str, count);
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("version.rs");
    fs::write(&dest_path, format!("pub const VERSION: &str = \"{}\";", version)).unwrap();
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
}
