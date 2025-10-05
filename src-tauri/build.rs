fn main() {
    println!("cargo:warning=HELLO FROM BUILD.RS!");
    tauri_build::build()
}
