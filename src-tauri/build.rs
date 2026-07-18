fn main() {
    tauri_build::build();

    #[cfg(target_os = "windows")]
    copy_python_dll_to_output();
}

/// 将 python313.dll 等复制到 target/{profile}/ 目录，使其与 exe 同目录。
/// Windows 加载器在进程启动时从 exe 所在目录查找 DLL，无需 /DELAYLOAD。
///
/// /DELAYLOAD 不可行：PyO3 引用了 Python 的数据符号（如 Py_NoneStruct），
/// 而 Windows 延迟加载只支持函数符号（LNK1194）。
#[cfg(target_os = "windows")]
fn copy_python_dll_to_output() {
    let Ok(out_dir) = std::env::var("OUT_DIR") else { return };
    let out_path = std::path::Path::new(&out_dir);

    // OUT_DIR 结构: target/{profile}/build/{crate}-{hash}/out
    // 向上 3 级到达 target/{profile}/
    let build_dir = match out_path
        .parent()  // out/
        .and_then(|p| p.parent())  // build-hash/
        .and_then(|p| p.parent())  // build/
    {
        Some(d) => d,
        None => return,
    };

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let python_dir = manifest_dir.join("binaries").join("python");

    if !python_dir.is_dir() {
        println!("cargo:warning=Python runtime not found at {:?}, run `pnpm setup:python` first", python_dir);
        return;
    }

    for dll in &["python313.dll", "python312.dll", "python311.dll", "python3.dll"] {
        let src = python_dir.join(dll);
        if src.exists() {
            let dst = build_dir.join(dll);
            match std::fs::copy(&src, &dst) {
                Ok(_) => println!("cargo:warning=Copied {} → {}", dll, dst.display()),
                Err(e) => println!("cargo:warning=Failed to copy {}: {}", dll, e),
            }
        }
    }
}
