fn main() {
    tauri_build::build();

    #[cfg(target_os = "windows")]
    {
        let dll_name = get_python_dll_name();
        println!("cargo:rustc-link-arg=/DELAYLOAD:{dll_name}");
    }
}

#[cfg(target_os = "windows")]
fn get_python_dll_name() -> String {
    use std::process::Command;
    for py_cmd in &["python", "python3"] {
        if let Ok(output) = Command::new(py_cmd)
            .args(["-c", "import sys; print(f'python{sys.version_info.major}{sys.version_info.minor}.dll')"])
            .output()
        {
            if output.status.success() {
                if let Ok(s) = String::from_utf8(output.stdout) {
                    let name = s.trim().to_string();
                    if !name.is_empty() {
                        println!("cargo:warning=Detected Python DLL: {name}");
                        return name;
                    }
                }
            }
        }
    }
    let fallback = "python311.dll".to_string();
    println!("cargo:warning=Could not detect Python version, using fallback: {fallback}");
    fallback
}
