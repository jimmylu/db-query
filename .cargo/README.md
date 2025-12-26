# Cargo 配置说明

## 网络超时问题解决

如果遇到 `libsqlite3-sys` 或其他依赖下载超时的问题，可以尝试以下解决方案：

### 方案 1: 使用国内镜像源（推荐）

编辑 `.cargo/config.toml`，取消注释镜像源配置：

```toml
[source.crates-io]
replace-with = "rustcc"

[source.rustcc]
registry = "https://crates.rustcc.cn"
```

### 方案 2: 增加超时时间

设置环境变量：

```bash
export CARGO_NET_TIMEOUT=300
```

### 方案 3: 手动下载依赖

如果网络问题持续，可以：

1. 使用代理或 VPN
2. 在另一个网络环境下先运行 `cargo fetch`
3. 将下载的依赖复制到本地

### 方案 4: 使用离线模式

如果依赖已经下载过，rust-analyzer 应该能够使用缓存的依赖。

## 当前配置

当前配置已设置：
- 重试次数：3次
- 使用 git CLI 进行 git 依赖获取（更可靠）

