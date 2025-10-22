## 技术栈与架构说明

### 目标
- 极小体积、极低资源占用的书签启动器
- 全局热键唤醒中央搜索框，模糊检索中文/英文/拼音
- 读取多个 Chromium 家族浏览器（Chrome/Edge/Opera/Opera GX）书签并保持更新

### 技术栈选择
- **Rust (2021)**: 零成本抽象、运行时开销低、单可执行文件体积小、跨平台能力强
- **wry**: 轻量级 WebView 容器，Win11 自带 WebView2 运行库，无需捆绑浏览器内核
- **windows-rs**: Windows 原生 API 绑定，用于注册全局热键
- **notify**: 文件系统事件监听（Windows 使用 ReadDirectoryChangesW）
- **serde/serde_json**: 高性能 JSON 解析（Chromium 书签文件为 JSON）
- **fuzzy-matcher（及自研权重排序）**: 轻量模糊匹配与多信号打分
- **once_cell / parking_lot**: 高性能并发原语与只读共享数据
- **simplelog/log**: 轻量日志
- 可选：**pinyin** crate（特性开关）用于更精准的汉字转拼音

### 模块划分
- **`main.rs`**: 进程入口与事件循环（wry EventLoop），初始化索引、创建 WebView、处理用户事件
- **`hotkey.rs`**: 全局快捷键注册与消息泵（Alt+Space），向主事件循环投递 Toggle 事件
- **`bookmarks.rs`**: 浏览器书签发现、解析与文件监听
  - 路径适配：Chrome/Edge 使用 `%LOCALAPPDATA%`，Opera/GX 使用 `%APPDATA%`
  - 解析 Chromium JSON 结构，收集 `title/url/browser/profile`
  - 监听书签文件变化，触发重建索引
- **`index.rs`**: 内存内索引与检索
  - 预计算字段：`title_lower`、`url_lower`、`title_pinyin_full`、`title_pinyin_initials`
  - 查询时多信号打分，排序截断（默认 30 条）
- **`ui.rs`**: 内嵌 HTML/CSS/JS 的沉浸式搜索界面
  - IPC 协议：`Search{query}`、`Open{url}`、`Hide`
  - 渲染结果列表、键盘交互（上下/回车/Esc）

### 运行时数据流
1. 启动时扫描所有受支持浏览器的书签，构建 `SearchIndex`
2. 用户按下全局热键 Alt+Space -> 窗口置顶显示并聚焦输入框
3. 输入触发节流（60ms），经 IPC 发送 `Search{query}` 到主进程
4. 主进程在索引中检索并回传结果数组，前端渲染
5. 用户回车或点击 -> IPC `Open{url}` -> 使用系统默认浏览器打开 -> 窗口隐藏
6. 后台文件监听到书签变化 -> 触发重新扫描与索引重建

### 性能与体积优化
- Release 配置：`lto=true, codegen-units=1, opt-level=z, strip, panic=abort`
- 索引预计算与只读共享，查询 O(N) 顺序扫描但数据量小（典型 <1e4）且分支可预测
- 空闲无需计时器，几乎不占用 CPU；仅输入与文件变更时工作
- 单可执行文件，无额外进程；WebView UI 仅在显示时存在渲染开销

### 兼容性
- Windows 10/11（已针对 Win11 优化；WebView2 在 Win11 自带，如 Win10 需安装运行库）
- 浏览器家族：Chrome / Edge / Opera / Opera GX（Chromium Bookmark 格式）
- 多 Profile 支持：默认 `Default` 与 `Profile *` 自动发现

### 安全与隐私
- 仅本地读取书签 JSON，不进行网络上传
- 打开 URL 使用系统默认浏览器，不拦截或注入
- 日志级别默认 Info，无敏感数据记录

### 局限与后续改进
- 拼音当前提供 fallback（ASCII）；可启用 `pinyin` 特性获得更准确转换
- 未支持 Firefox（需解析 `places.sqlite`）；可作为增强项
- 全局热键冲突时需手动修改配置或 UI 引导重设
- 检索算法可替换为带索引的倒排或 `tantivy`，在极大量书签时更高效

### 打包与分发
- 使用 `cargo build --release` 直接生成小体积 exe
- 可选 `cargo-bundle` 或 `wix` 生成安装包（视分发渠道决定）
- 自启动：快捷方式放入 Startup 或写入 `HKCU\\...\\Run`
