fn main() {
    // This triggers Tauri's internal bundler before compiling the final Rust binary.
    // It guarantees that our React UI and WebAssembly core are securely packed 
    // inside the final .exe or .app file, requiring no external downloads for the user.
    tauri_build::build()
}