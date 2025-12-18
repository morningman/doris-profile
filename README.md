# Doris Profile Analyzer

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Vue.js](https://img.shields.io/badge/vue.js-3.x-green.svg)](https://vuejs.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**A professional tool for analyzing Apache Doris query profiles with intelligent performance insights**

[English](#english) | [中文](#中文)

</div>

---

## English

### Overview

Doris Profile Analyzer is a powerful tool designed to parse, analyze, and visualize Apache Doris query profiles. It provides accurate performance metrics, intelligent bottleneck detection, and actionable optimization suggestions.

### Key Features

- **Accurate Parsing**: Parse Doris query profile with detailed metrics extraction
- **Smart Diagnostics**: Automatic performance bottleneck identification
- **Interactive Visualization**: DAG-based execution plan visualization
- **Optimization Suggestions**: Automated recommendations based on best practices
- **High Performance**: Optimized for large files with efficient memory usage
- **Modern UI**: Web interface with file upload and text paste support

### Quick Start

#### Prerequisites

- Rust 1.70+
- Node.js 16+ (only for building from source)
- System dependencies: gcc, pkg-config, openssl-dev

#### Automated Setup (Recommended)

```bash
# Clone the repository
git clone https://github.com/your-org/doris-profile-analyzer.git
cd doris-profile-analyzer

# Install system dependencies automatically
./install-deps.sh

# Build single executable with embedded frontend
make build
```

#### Manual Installation

If you prefer to install dependencies manually or the script doesn't work for your system, see **[BUILD.md](BUILD.md)** for detailed instructions.

This will create a single executable at `build/doris-profile-analyzer` with all frontend assets embedded.

#### Run

```bash
# Run with default settings (port 3030)
./build/doris-profile-analyzer

# Run with custom port
./build/doris-profile-analyzer --port 8080

# Run with custom host and port
./build/doris-profile-analyzer --host 127.0.0.1 --port 8080

# Show help
./build/doris-profile-analyzer --help
```

**Access:**
- Web UI & API: http://localhost:3030 (or your custom port)

### Usage

#### Upload Profile

- **File Upload**: Supports `.txt`, `.log`, `.profile` formats (max 50MB)
- **Text Paste**: Directly paste profile content
- **Drag & Drop**: Drag files to upload area

#### View Analysis

- **Execution Tree**: Interactive DAG visualization
- **Hotspots**: Automatically identified performance bottlenecks
- **Suggestions**: Optimization recommendations
- **Performance Score**: Overall performance assessment

#### API Examples

**Health Check:**
```bash
curl http://localhost:3030/health
```

**Analyze Text:**
```bash
curl -X POST http://localhost:3030/api/analyze \
  -H "Content-Type: application/json" \
  -d '{"profile_text": "Your profile content"}'
```

**Analyze File:**
```bash
curl -X POST http://localhost:3030/api/analyze-file \
  -F "file=@/path/to/profile.txt"
```

### Architecture

```
backend/src/
├── api/              # HTTP API layer
├── parser/           # Profile parser
├── analyzer/         # Performance analyzer
├── models.rs         # Data models
└── constants.rs      # Configuration constants

frontend/src/
├── components/       # Vue components
├── views/            # Page views
├── store/            # State management
└── utils/            # Utility functions
```

### License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 中文

### 概述

Doris Profile 分析器是一款专业的查询性能分析工具，用于解析、分析和可视化 Apache Doris 查询 Profile。提供精准的性能指标、智能瓶颈检测和可执行的优化建议。

### 核心特性

- **精准解析**：解析 Doris 查询 Profile，提取详细指标
- **智能诊断**：自动识别执行计划中的性能瓶颈
- **可视化展示**：基于 DAG 的交互式执行计划可视化
- **优化建议**：基于最佳实践的自动化建议
- **高性能**：支持大文件解析，内存使用优化
- **现代界面**：Web 界面，支持文件上传和文本粘贴

### 快速开始

#### 环境要求

- Rust 1.70+
- Node.js 18+（仅构建时需要）
- make

#### 安装与构建

```bash
# 克隆项目
git clone https://github.com/your-org/doris-profile-analyzer.git
cd doris-profile-analyzer

# 构建单一可执行文件（内嵌前端资源）
make build
```

这将在 `build/doris-profile-analyzer` 生成一个包含所有前端资源的单一可执行文件。

#### 运行

```bash
# 使用默认配置运行（端口 3030）
./build/doris-profile-analyzer

# 自定义端口运行
./build/doris-profile-analyzer --port 8080

# 自定义主机和端口
./build/doris-profile-analyzer --host 127.0.0.1 --port 8080

# 查看帮助
./build/doris-profile-analyzer --help
```

**访问：**
- Web 界面 & API：http://localhost:3030（或您自定义的端口）

### 使用指南

#### 上传 Profile

- **文件上传**：支持 `.txt`、`.log`、`.profile` 格式（最大 50MB）
- **文本粘贴**：直接粘贴 Profile 文本内容
- **拖拽上传**：拖拽文件到上传区域

#### 查看分析结果

- **执行树**：交互式 DAG 图展示
- **热点问题**：自动识别的性能瓶颈
- **优化建议**：基于最佳实践的建议
- **性能评分**：整体性能评估

### 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

<div align="center">

**Made for Apache Doris Community**

</div>

