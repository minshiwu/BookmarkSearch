# Bookmark Launcher (Windows)

一个极简、极速的书签启动器：全局快捷键唤醒、中央搜索框、支持 Chrome / Edge / Opera / Opera GX，多语言（中文/英文/拼音）模糊检索，点击使用系统默认浏览器打开。

## 功能
- 全局热键 Alt+Space 唤醒/隐藏
- 即时搜索：标题、URL、拼音全拼与首字母
- 支持多个浏览器与多个用户资料夹（Profile）
- 监听书签变更，自动刷新索引
- 轻量安装与极低资源占用

## 安装与运行（Windows 10/11）
1. 安装 Rust (MSVC 工具链)：`https://rustup.rs`。
2. 构建发布版：
   ```bash
   cargo build --release
   ```
3. 可执行文件在：`target/release/bookmark_launcher.exe`。
4. 双击运行，或创建桌面/开始菜单快捷方式。

### 自启动
- 将 `bookmark_launcher.exe` 的快捷方式放入：`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`。
- 或添加注册表项（当前用户）：
  1. 运行 `regedit`。
  2. 定位到 `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`。
  3. 新建字符串值，名称自定，数据为 exe 的完整路径。

## 使用
- 按 Alt+Space 唤醒/隐藏
- 输入中文、英文或拼音；上下键选择；Enter 打开；Esc 关闭
- 点击结果亦可打开

## 体积与性能
- 单文件 exe（不含 WebView2 运行库，Win11 自带）
- 开启 LTO/strip，体积尽可能小
- 仅在输入/文件变更时工作；空闲时几乎零资源占用

## 支持的浏览器书签路径（默认）
- Chrome: `%LOCALAPPDATA%/Google/Chrome/User Data/*/Bookmarks`
- Edge: `%LOCALAPPDATA%/Microsoft/Edge/User Data/*/Bookmarks`
- Opera: `%APPDATA%/Opera Software/Opera Stable/Bookmarks`
- Opera GX: `%APPDATA%/Opera Software/Opera GX Stable/Bookmarks`

## 注意
- 首次运行会索引默认 Profile；若你有多个 Profile，会自动发现 `Profile *` 目录
- 热键与其它软件冲突时可修改源码中的组合键（`hotkey.rs`）
- 拼音转换默认启用 fallback（ASCII）；若需更精准拼音，可启用 `pinyin` 功能并添加依赖

## 启用拼音库（可选）
在 `Cargo.toml` 中加入 pinyin 依赖，并启用 feature：
```toml
[dependencies]
pinyin = "0.10"

[features]
default = []
pinyin = []
```
并在构建时：
```bash
cargo build --release --features pinyin
```

## 许可证
MIT
