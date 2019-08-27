fn main() {
    let rust_output = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("ffi.rs");

    let output = generator::generate(
        &std::env::current_dir().unwrap().as_path().join("src/ffi.rs"),
        "rust_unit",
        false,
    );

    use std::io::Write;
    std::fs::File::create(rust_output.clone())
        .unwrap()
        .write_all(output.as_bytes())
        .unwrap();

    let _ = ::std::process::Command::new("rustfmt")
        .arg(rust_output.display().to_string())
        .spawn();

    assert!(rust_output.exists());
}
