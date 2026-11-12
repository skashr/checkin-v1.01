# 每日打卡 · 桌面版（EXE）

把「每日打卡」做成 Windows 桌面程序，数据存成一个 JSON 文件，可以放进网盘同步目录实现多设备同步。

- 前端：纯 HTML/CSS/JS（无框架、无构建步骤），界面与网页版一致。
- 后端：Tauri v2 + Rust，负责把数据读写到本地文件、系统托盘、开机自启、单实例。
- 数据：`checkin-data.json`（默认在应用数据目录，可改到任意网盘目录）。

---

## 一、用 GitHub 云端编译出 EXE（推荐，本机不用装任何环境）

### 第 1 步：准备一个 GitHub 账号和仓库

1. 打开 https://github.com ，注册账号（免费）。
2. 点右上角 **+** → **New repository**：
   - Repository name 填 `checkin`（随意）。
   - 选 **Private**（私有，仅自己可见）。
   - 其它默认，点 **Create repository**。

### 第 2 步：上传本项目代码

进到刚建的仓库页面，点 **Add file** → **Upload files**，然后**把本 `checkin-exe` 文件夹里的所有内容**拖进浏览器上传框（保持目录结构，注意要包含 `.github` 和 `src-tauri` 和 `ui` 这三个文件夹），点 **Commit changes**。

> 也可以用 GitHub Desktop（图形界面）更方便地拖拽上传，无需命令行。

### 第 3 步：等它自动构建

上传后，GitHub 会自动开始构建。点仓库顶部的 **Actions** 标签，能看到一个叫 **构建 Windows 安装包** 的任务在跑（约 5~10 分钟）。

### 第 4 步：下载安装包

构建完成后，进入那个任务，底部 **Artifacts** 里有个 **每日打卡-安装包**，点下载，解压后里面有：
- `*.exe`（NSIS 安装程序，推荐）
- `*.msi`（MSI 安装程序）

双击 `.exe` 安装即可使用。安装后程序在开始菜单和桌面出现，数据文件在：
`C:\Users\<你>\AppData\Roaming\com.dailycheckin.app\checkin-data.json`

---

## 二、设置网盘同步（多设备共享数据）

在程序里：**设置 → 数据文件夹（网盘同步）→ 选择文件夹**，选到你的网盘同步目录，例如：

- OneDrive：`C:\Users\<你>\OneDrive\打卡数据`
- 坚果云：你设置的同步文件夹
- Dropbox：你的 Dropbox 文件夹

之后程序的所有打卡数据都会写入那个目录里的 `checkin-data.json`，由网盘自动同步到其它电脑；另一台电脑装同样程序、同样把数据文件夹指到网盘目录即可共享。

> 说明：网盘同步是「以文件为准」的简单同步，适合单人多设备、不同时操作。若两台电脑**同时**修改，可能产生冲突副本，以网盘保留的那份为准。

---

## 三、本地也能自己编译（可选，需要装环境）

前提：Rust 工具链 + Visual Studio C++ Build Tools + WebView2（Win11 已内置）。

```bash
npm install
npm run build          # 等同于 tauri build
```

产物在 `src-tauri/target/release/bundle/`。

---

## 四、功能一览

- 日 / 周 / 月 / 设置 四个视图；任务设置一次后每天自动出现，每天点 ✅/❌ 打卡。
- 完成率、周/月统计、连续打卡天数，跨月/跨年/闰年正确。
- 系统托盘：关闭窗口最小化到托盘；重复启动聚焦已有窗口。
- 开机自启（设置里开关）；每日提醒（系统通知）。
- 数据备份：导出/导入 JSON。

## 五、数据格式

数据文件 `checkin-data.json` 结构：

```json
{
  "tasks":    [ { "id", "name", "color", "enabled", "sortOrder", "createdAt" } ],
  "checkins": { "任务id|YYYY-MM-DD": 0 或 1 },
  "settings": { "reminderEnabled", "reminderTime", "lastReminderDate" }
}
```

与网页版导出的 JSON 备份完全兼容，可互相导入。
