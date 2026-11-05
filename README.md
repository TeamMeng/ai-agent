# ai-agent

Rust 2024 项目模板，包含 anyhow 错误处理、tracing 日志、Clippy、nextest 和 pre-commit。

## 快速开始

需要安装 Rust stable。

```bash
cargo install --locked cargo-generate cargo-nextest
cargo generate TeamMeng/template
cd <project-name>
cargo run
```

## 开发命令

| 命令 | 用途 |
| --- | --- |
| `cargo run` | 运行程序 |
| `cargo fmt --all` | 格式化代码 |
| `cargo lint` | Clippy 检查，警告视为错误 |
| `cargo nt` | 使用 nextest 运行测试 |

`cargo lint` 和 `cargo nt` 是 `.cargo/config.toml` 中定义的别名，分别对应：

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
```

## 提交前检查（可选）

先安装 pipx，再执行：

```bash
pipx install pre-commit
cargo install --locked typos-cli cargo-deny
pre-commit install
```

提交时自动检查文件规范、拼写、代码格式、依赖、Clippy 和测试。
需要已安装上述工具及快速开始中的 `cargo-nextest`。

手动检查全部文件：

```bash
pre-commit run --all-files
```

## License

[MIT](LICENSE)
