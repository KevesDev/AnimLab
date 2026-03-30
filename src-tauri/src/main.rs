// AAA DISTRIBUTION RULE: 
// This macro tells the Windows OS to hide the command prompt terminal. 
// It is strictly ignored during debug mode so we can see error logs, 
// but it physically removes the terminal window from the final Release build.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::process;

fn main() {
    // Wraps the entire OS boot sequence in a secure Result to prevent unhandled panics.
    if let Err(init_error) = boot_desktop_shell() {
        eprintln!("FATAL OS ERROR: AnimLab failed to initialize the native desktop shell.");
        eprintln!("System Diagnostics: {}", init_error);
        
        // Exits the application cleanly with a standard error code rather than panicking memory.
        process::exit(1);
    }
}

/// Initializes the Tauri windowing system and binds the webview context.
fn boot_desktop_shell() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        // The setup hook provides access to the raw OS application instance, 
        // allowing us to initialize native file loggers or disk access registries.
        .setup(|_app| {
            // Native initialization logic will be mounted here.
            Ok(())
        })
        .run(tauri::generate_context!())?;
        
    Ok(())
}