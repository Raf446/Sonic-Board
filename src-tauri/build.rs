use std::env;
use std::path::PathBuf;

fn main() {
    tauri_build::build();
    if let Ok(out_dir) = env::var("OUT_DIR") {
        let out_path = PathBuf::from(&out_dir);
        let res = out_path.join("libresource.a");
        if res.exists() {
            println!("cargo:rustc-link-arg=-Wl,--whole-archive");
            println!("cargo:rustc-link-arg={}", res.display());
            println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");
        }
    }
}
