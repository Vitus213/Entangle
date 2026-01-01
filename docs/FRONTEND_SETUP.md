# Entangle 前端构建配置说明

> 本文档说明如何使用 Nix 管理前端构建环境

---

## ✅ 已配置内容

### 1. Nix Flake 配置 (`flake.nix`)

#### Rust 工具链
```nix
rustToolchainWithWasm = fenix.packages.${system}.combine [
  rustToolchain
  fenix.packages.${system}.targets.wasm32-unknown-unknown.latest.rust-std
];
```

**包含:**
- ✅ Rust 稳定版工具链
- ✅ wasm32-unknown-unknown target
- ✅ rust-analyzer
- ✅ clippy, rustfmt

#### 前端构建工具
```nix
buildInputs = [
  trunk                 # Leptos/WASM 打包器
  wasm-bindgen-cli      # Rust/WASM 绑定生成
  binaryen              # wasm-opt 优化器
];
```

#### 环境变量
```nix
TRUNK_SERVE_PORT = "8080";
TRUNK_SERVE_ADDRESS = "127.0.0.1";
```

---

## 🚀 快速开始

### 1. 进入开发环境

```bash
# 使用 Nix
nix develop

# 或使用 direnv（自动加载）
direnv allow .
```

进入环境后会显示:
```
🚀 Entangle Development Environment
==================================
Rust: rustc 1.x.x
Cargo: cargo 1.x.x
Trunk: trunk 0.x.x

📦 WASM Target: wasm32-unknown-unknown
   ✅ Available via Nix
```

### 2. 启动前端开发

```bash
# 方式 1: 使用 trunk（推荐）
cd frontend
trunk serve

# 方式 2: 使用脚本
./scripts/build_and_serve_frontend.sh
```

### 3. 全栈开发

```bash
# Terminal 1: 后端
just dev

# Terminal 2: 前端
cd frontend && trunk serve
```

访问 http://localhost:8080

---

## 📦 构建命令

### 开发模式
```bash
cd frontend

# 启动开发服务器（热重载）
trunk serve

# 自定义端口
trunk serve --port 3001

# 监听所有接口（远程访问）
trunk serve --address 0.0.0.0
```

### 生产构建
```bash
cd frontend

# 发布版本（优化）
trunk build --release

# 输出目录
ls dist/
# -> index.html
# -> entangle_frontend.js
# -> entangle_frontend_bg.wasm
```

---

## 🔧 配置文件

### frontend/Trunk.toml
```toml
[build]
target = "index.html"
release = true

[watch]
ignore = ["dist", "target"]

[serve]
address = "127.0.0.1"
port = 8080
```

### frontend/Cargo.toml
```toml
[profile.release]
opt-level = 'z'       # 优化大小
lto = true            # 链接时优化
codegen-units = 1     # 单编译单元
```

---

## 🛠️ 工具说明

### Trunk
- **作用**: WASM 应用打包和开发服务器
- **功能**:
  - 自动编译 Rust -> WASM
  - 运行 wasm-bindgen
  - 注入 JavaScript 加载器
  - 热重载开发服务器
  - 生产优化（wasm-opt）

### wasm-bindgen
- **作用**: 生成 Rust/JavaScript 绑定
- **自动处理**: Trunk 自动调用，无需手动运行

### wasm-opt
- **作用**: WASM 二进制优化
- **效果**: 减小 WASM 文件大小 30-50%
- **使用**: trunk build --release 时自动运行

---

## 📁 目录结构

```
frontend/
├── Cargo.toml              # Rust 依赖
├── Trunk.toml              # Trunk 配置
├── index.html              # HTML 模板
├── src/
│   ├── main.rs            # 入口点
│   └── lib.rs             # 应用逻辑
├── dist/                  # 构建输出（自动生成）
│   ├── index.html
│   ├── *.js
│   └── *.wasm
└── target/                # Rust 编译缓存
```

---

## 🐛 常见问题

### Q1: wasm32 target 未安装?

**解决**: 使用 Nix 环境，target 自动提供

```bash
nix develop
# wasm32-unknown-unknown 自动可用
```

### Q2: trunk 命令找不到?

**解决**: 确保在 Nix 环境中

```bash
# 检查
which trunk
# -> /nix/store/xxx-trunk-x.x.x/bin/trunk

# 如果没有，重新进入
nix develop
```

### Q3: 构建很慢？

**优化建议**:
1. **使用开发构建** (不加 --release)
2. **启用增量编译** (已默认启用)
3. **使用 trunk serve** (只重新编译改动部分)

### Q4: WASM 文件太大？

**解决**:
1. 使用 `trunk build --release`
2. 检查 Cargo.toml profile.release 配置
3. 启用服务器 gzip/brotli 压缩

---

## 📊 性能指标

### 典型构建时间
- **开发构建**: ~10-30 秒
- **生产构建**: ~30-60 秒
- **热重载**: <2 秒

### 产物大小（release 优化后）
- **WASM**: ~200-500 KB
- **JS 胶水代码**: ~10-20 KB
- **总计**: ~210-520 KB
- **Gzip 后**: ~60-150 KB

---

## 🔄 CI/CD 集成

### GitHub Actions 示例

```yaml
name: Build Frontend

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Nix
        uses: cachix/install-nix-action@v22
        with:
          extra_nix_config: |
            experimental-features = nix-command flakes

      - name: Build Frontend
        run: |
          nix develop --command bash -c "
            cd frontend
            trunk build --release
          "

      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: frontend-dist
          path: frontend/dist/
```

---

## 🚀 部署

### 静态文件部署

构建后的 `frontend/dist/` 可部署到任何静态托管:

```bash
# 1. 构建
cd frontend && trunk build --release

# 2. 部署到服务器
rsync -avz dist/ user@server:/var/www/entangle/

# 或使用 S3/CDN
aws s3 sync dist/ s3://your-bucket/
```

### Nginx 配置

```nginx
server {
    listen 80;
    server_name entangle.example.com;

    root /var/www/entangle;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # WASM MIME type
    location ~* \.wasm$ {
        types {
            application/wasm wasm;
        }
    }

    # API 代理
    location /api {
        proxy_pass http://localhost:3000;
    }
}
```

---

## 📚 更多资源

- [前端开发指南](FRONTEND_GUIDE.md) - 详细开发文档
- [Leptos 文档](https://leptos.dev/)
- [Trunk 文档](https://trunkrs.dev/)
- [Nix Flakes 手册](https://nixos.wiki/wiki/Flakes)

---

*配置完成日期: 2026-01-01*
