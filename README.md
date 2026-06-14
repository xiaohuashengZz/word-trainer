# 📚 英语单词学习助手

基于间隔重复算法（SM-2）的英语单词复习工具，采用 Tauri + SolidJS 技术栈开发。

## ✨ 功能特性

- **智能复习提醒** - 可设置 5 分钟至 3 小时的定时提醒间隔
- **间隔重复算法** - 采用 SM-2 算法优化复习间隔，科学对抗遗忘曲线
- **多种复习模式** - 支持跳过、重置单词状态
- **详细统计** - 实时显示学习进度、正确率等数据
- **系统托盘** - 最小化到托盘，随时快速访问
- **本地存储** - SQLite 数据库，数据安全可控

## 🚀 快速开始

### 前置要求

- Node.js 18+
- Rust 1.70+
- Windows 10/11

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建发布

```bash
npm run tauri build
```

构建完成后，可执行文件位于：
- `src-tauri/target/release/word-trainer.exe` - 直接运行的可执行文件
- `src-tauri/target/release/bundle/nsis/*.exe` - Windows 安装程序

## 📖 使用说明

### 添加单词

1. 点击「添加」标签
2. 输入英文单词
3. 添加释义（支持多个词性和释义）
4. 可选：添加音标和发音链接

### 复习单词

1. 点击「复习」标签开始学习
2. 系统展示英文单词，尝试回忆中文释义
3. 输入答案后按回车提交
4. 系统判断正确与否，显示正确答案
5. 点击「下一题」继续

### 设置提醒

1. 点击右上角 ⚙️ 进入设置
2. 开启「复习提醒」开关
3. 选择提醒间隔（5分钟～3小时）
4. 应用将定时弹出复习窗口

## 🏗️ 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | SolidJS |
| 构建工具 | Vite |
| 状态管理 | Nanostores |
| 后端框架 | Tauri 2.0 |
| 编程语言 | Rust |
| 数据库 | SQLite |
| 测试 | Vitest |

## 📁 项目结构

```
word-trainer/
├── src/                    # 前端源码
│   ├── components/         # 组件
│   ├── stores/             # 状态管理
│   ├── types/              # TypeScript 类型
│   └── __tests__/          # 测试文件
├── src-tauri/             # Tauri 后端
│   ├── src/
│   │   ├── commands/       # Tauri 命令
│   │   ├── domain/         # 领域模型
│   │   ├── infrastructure/ # 基础设施（数据库）
│   │   └── algorithm/      # SM-2 算法
│   └── core/               # 核心库
└── public/                 # 静态资源
```

## 🧪 测试

```bash
npm run test      # 运行测试（监听模式）
npm run test:run  # 运行测试（单次）
```

## 📝 License

MIT License
