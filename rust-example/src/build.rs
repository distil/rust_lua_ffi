use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let env_out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&env_out_dir);
    let rust_output = out_dir.join("ffi.rs");

    let output = Command::new("lua-c-ffi-marshalling")
        .args(&["--input", env::current_dir().unwrap().as_path().join("src/ffi.rs").to_str().unwrap()])
        .args(&["--output", rust_output.to_str().unwrap()])
        .args(&["--library-name", "rust_example"])
        .output()
        .expect("lua-c-ffi-marshalling failed");
    print!("--- stdout\n{}\n", String::from_utf8_lossy(&output.stdout).as_ref());
    print!("--- stderr\n{}\n", String::from_utf8_lossy(&output.stderr).as_ref());
    assert!(output.status.success());
    assert!(rust_output.exists());
}
