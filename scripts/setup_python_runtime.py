#!/usr/bin/env python3
"""
下载并配置嵌入式 Python 运行时用于 Windows 分发。

用法:
  python scripts/setup_python_runtime.py           # 使用本地 Python 版本
  python scripts/setup_python_runtime.py 3.11.10   # 指定版本
  python scripts/setup_python_runtime.py --clean   # 仅清理
"""

import argparse
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parent.parent
PYTHON_DIR = PROJECT_ROOT / "src-tauri" / "binaries" / "python"
CORE_DIR = PROJECT_ROOT / "core"
REQUIREMENTS_TXT = PROJECT_ROOT / "requirements.txt"
REQUIREMENTS_LOCK = PROJECT_ROOT / "requirements.lock"

RELEASE_TAG = "20241016"
BASE_URL = f"https://github.com/indygreg/python-build-standalone/releases/download/{RELEASE_TAG}"

# python-build-standalone 每个 release 只包含特定 micro 版本。
# 同一 minor 版本 (如 3.11.x) 的 DLL ABI 完全兼容，可混用。
VERSION_MAP = {
    "3.10": "3.10.15",
    "3.11": "3.11.10",
    "3.12": "3.12.7",
    "3.13": "3.13.0",
}


def get_local_python_version() -> str:
    result = subprocess.run(
        [sys.executable, "-c",
         "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}')"],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print("ERROR: 无法检测本地 Python 版本")
        sys.exit(1)
    return result.stdout.strip()


def resolve_version(requested: str) -> str:
    """将用户请求的版本映射到 python-build-standalone 中实际存在的版本"""
    parts = requested.split(".")
    if len(parts) >= 2:
        major_minor = f"{parts[0]}.{parts[1]}"
        if major_minor in VERSION_MAP:
            resolved = VERSION_MAP[major_minor]
            if resolved != requested:
                print(f"版本映射: {requested} → {resolved} (ABI 兼容)")
            return resolved
    # 如果不在映射表中，尝试直接使用（可能存在于新 release 中）
    print(f"WARNING: {requested} 不在已知版本映射中，直接尝试下载")
    return requested


def download_python(version: str) -> None:
    """下载 python-build-standalone for Windows x64"""
    resolved = resolve_version(version)
    archive_name = f"cpython-{resolved}+{RELEASE_TAG}-x86_64-pc-windows-msvc-install_only.tar.gz"
    url = f"{BASE_URL}/{archive_name}"

    print(f"下载: {url}")
    tmp_dir = Path(tempfile.mkdtemp(prefix="douyin-python-"))
    archive_path = tmp_dir / archive_name

    try:
        _download_file(url, archive_path)
        _verify_tarball(archive_path, archive_name)
    except Exception as e:
        print(f"ERROR: 下载失败: {e}")
        print()
        print(f"可用的 Python 版本 (release {RELEASE_TAG}):")
        for k, v in VERSION_MAP.items():
            print(f"  {k}.x → {v}")
        print()
        print("如需其他版本，修改脚本中的 RELEASE_TAG 并更新 VERSION_MAP。")
        print(f"Release 页面: https://github.com/indygreg/python-build-standalone/releases/tag/{RELEASE_TAG}")
        sys.exit(1)

    print(f"解压到: {PYTHON_DIR}")
    if PYTHON_DIR.exists():
        shutil.rmtree(PYTHON_DIR)
    PYTHON_DIR.mkdir(parents=True, exist_ok=True)

    with tarfile.open(archive_path, "r:gz") as tar:
        members = tar.getmembers()
        prefix = _find_common_prefix(members)
        for member in members:
            if prefix:
                rel = member.name[len(prefix):].lstrip("/")
            else:
                rel = member.name
            if not rel:
                continue
            member.name = rel
            tar.extract(member, PYTHON_DIR)

    shutil.rmtree(tmp_dir)
    print(f"Python {resolved} 运行时已就绪: {PYTHON_DIR}")
    _list_key_files()


def _download_file(url: str, dest: Path) -> None:
    """下载文件（带进度和重试）"""
    import time

    max_retries = 3
    for attempt in range(max_retries):
        try:
            print(f"  下载中... (尝试 {attempt + 1}/{max_retries})")
            req = urllib.request.Request(url, headers={"User-Agent": "DouyinCrawler/1.0"})
            with urllib.request.urlopen(req, timeout=60) as resp:
                total = int(resp.headers.get("Content-Length", 0))
                downloaded = 0
                with open(dest, "wb") as f:
                    while True:
                        chunk = resp.read(1024 * 1024)
                        if not chunk:
                            break
                        f.write(chunk)
                        downloaded += len(chunk)
                        if total > 0:
                            pct = downloaded * 100 // total
                            mb = downloaded / (1024 * 1024)
                            total_mb = total / (1024 * 1024)
                            print(f"\r  {pct}% ({mb:.1f}/{total_mb:.1f} MB)", end="", flush=True)
                print()
            return
        except Exception as e:
            if attempt < max_retries - 1:
                print(f"  失败，{2}s 后重试: {e}")
                time.sleep(2)
            else:
                raise


def _verify_tarball(path: Path, name: str) -> None:
    """验证下载的 tar.gz 文件完整性"""
    if not path.exists():
        raise RuntimeError(f"文件不存在: {path}")
    size_mb = path.stat().st_size / (1024 * 1024)
    if size_mb < 1:
        raise RuntimeError(f"文件过小 ({size_mb:.1f} MB)，可能不完整")
    try:
        with tarfile.open(path, "r:gz") as tar:
            members = tar.getmembers()
        print(f"  验证通过: {len(members)} 个文件, {size_mb:.1f} MB")
    except Exception as e:
        raise RuntimeError(f"tar.gz 损坏: {e}")


def _find_common_prefix(members: list) -> str:
    """查找 tar 成员的公共前缀目录"""
    names = [m.name.strip("/") for m in members if not m.name.endswith("/")]
    if not names:
        return ""
    prefix = names[0].split("/")[0] if "/" in names[0] else ""
    for name in names[1:]:
        parts = name.split("/")
        if not parts or parts[0] != prefix:
            return ""
    return prefix + "/"


def _list_key_files() -> None:
    for f in ["python.exe", "python3.dll", "python311.dll", "python312.dll", "python313.dll"]:
        p = PYTHON_DIR / f
        if p.exists():
            print(f"  ✓ {f}")
    lib = PYTHON_DIR / "Lib"
    if lib.exists():
        py_files = len(list(lib.rglob("*.py")))
        print(f"  ✓ Lib/ ({py_files} 个 .py 文件)")
    sites = lib / "site-packages"
    if sites.exists() and any(sites.iterdir()):
        print(f"  ✓ Lib/site-packages/ ({len(list(sites.iterdir()))} 个包)")


def install_dependencies() -> None:
    target = PYTHON_DIR / "Lib" / "site-packages"
    target.mkdir(parents=True, exist_ok=True)

    cmd = [
        sys.executable, "-m", "pip", "install",
        "--target", str(target),
        "-r", str(REQUIREMENTS_TXT),
    ]
    if REQUIREMENTS_LOCK.exists():
        cmd.extend(["-c", str(REQUIREMENTS_LOCK)])

    print(f"安装依赖: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=False)
    if result.returncode != 0:
        print("ERROR: pip install 失败")
        sys.exit(1)

    # 交叉编译场景：用 Windows 原生 wheel 替换 Linux .so 文件
    _fix_native_packages_for_windows(target)

    print("依赖安装完成")
    _list_key_files()


def _fix_native_packages_for_windows(site_packages: Path) -> None:
    """如果当前不在 Windows 上，用 win_amd64 wheel 替换原生包。

    pip install --target 在 Linux 上会安装 Linux 原生 .so 文件。
    以下包有原生扩展，在 Windows 上需要 .pyd 文件：
    - pydantic_core (Rust/PyO3)
    - pycryptodomex (C extension)
    - yaml/PyYAML (C extension, 有 pure-Python fallback)
    """
    if sys.platform == "win32":
        return  # 在 Windows 上不需要修复

    py_version = f"{sys.version_info.major}{sys.version_info.minor}"
    platforms = [
        f"win_amd64",
        f"win32",
    ]

    native_packages = [
        "pydantic-core",
        "pycryptodomex",
        "pyyaml",
    ]

    # 读取 lock 文件中的版本约束
    constraints = {}
    if REQUIREMENTS_LOCK.exists():
        for line in REQUIREMENTS_LOCK.read_text().splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "==" in line:
                pkg, ver = line.split("==", 1)
                constraints[pkg.strip()] = ver.strip()

    # 查找 pip install 实际安装的版本
    installed_versions = {}
    for dist_info in site_packages.glob("*.dist-info"):
        name = dist_info.name.replace(".dist-info", "")
        if "-" in name:
            parts = name.rsplit("-", 1)
            pkg_name = parts[0]
            pkg_version = parts[1]
            # 规范化名称
            normalized = pkg_name.lower().replace("_", "-")
            installed_versions[normalized] = (pkg_name, pkg_version)

    for pkg in native_packages:
        normalized = pkg.lower().replace("_", "-")
        if normalized not in installed_versions:
            continue

        pkg_name, pkg_version = installed_versions[normalized]
        pkg_with_ver = f"{pkg_name}=={pkg_version}"

        for plat in platforms:
            print(f"  下载 Windows wheel: {pkg_with_ver} ({plat})")
            result = subprocess.run(
                [
                    sys.executable, "-m", "pip", "download",
                    "--dest", str(site_packages.parent / "_win_wheels"),
                    "--platform", plat,
                    "--python-version", py_version,
                    "--no-deps",
                    "--only-binary", ":all:",
                    pkg_with_ver,
                ],
                capture_output=True, text=True,
            )
            if result.returncode == 0:
                # 找到 wheel，解压
                wheels_dir = site_packages.parent / "_win_wheels"
                for whl in wheels_dir.glob("*.whl"):
                    print(f"  解压: {whl.name}")
                    with zipfile.ZipFile(whl) as z:
                        z.extractall(site_packages)
                # 清理旧 Linux .so 文件
                _clean_linux_native(site_packages)
                import shutil
                shutil.rmtree(wheels_dir, ignore_errors=True)
                break
            else:
                print(f"  {plat} 失败: {result.stderr.strip()}")
        else:
            print(f"  WARNING: 无法下载 {pkg_with_ver} 的 Windows wheel，"
                  f"该包在 Windows 上可能无法正常工作")


def _clean_linux_native(site_packages: Path) -> None:
    """清理 Linux .so 原生文件，避免与 Windows .pyd 冲突"""
    removed = 0
    for so_file in site_packages.rglob("*.so"):
        so_file.unlink()
        removed += 1
    if removed:
        print(f"  清理了 {removed} 个 Linux .so 文件")


def clean_pycache() -> None:
    count = 0
    for pycache in CORE_DIR.rglob("__pycache__"):
        if pycache.is_dir():
            shutil.rmtree(pycache)
            count += 1
    print(f"清理了 {count} 个 __pycache__ 目录")


def clean_python_dir() -> None:
    if PYTHON_DIR.exists():
        shutil.rmtree(PYTHON_DIR)
        print(f"已删除: {PYTHON_DIR}")
    PYTHON_DIR.mkdir(parents=True, exist_ok=True)


def main():
    parser = argparse.ArgumentParser(description="配置嵌入式 Python 运行时")
    parser.add_argument("version", nargs="?", help="Python 版本号 (如 3.11.10)")
    parser.add_argument("--no-deps", action="store_true", help="跳过依赖安装")
    parser.add_argument("--clean", action="store_true", help="仅清理已下载的 Python")
    args = parser.parse_args()

    if args.clean:
        clean_python_dir()
        clean_pycache()
        return

    version = args.version or get_local_python_version()
    print(f"目标 Python 版本: {version}")

    download_python(version)

    if not args.no_deps:
        install_dependencies()

    clean_pycache()

    print()
    print("=" * 60)
    print("配置完成！现在可以运行:")
    print("  pnpm tauri build")
    print("=" * 60)


if __name__ == "__main__":
    main()
