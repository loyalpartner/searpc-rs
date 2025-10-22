# GitHub Actions 发布指南

## CI/CD 工作流

### 1. 持续集成 (CI)

**触发条件**：
- Push 到 master 分支
- 创建 Pull Request

**执行内容**：
- 代码格式检查 (`cargo fmt`)
- Clippy 检查 (`cargo clippy`)
- 构建 (`cargo build`)
- 运行测试 (`cargo test`)
- 生成文档 (`cargo doc`)
- MSRV 检查（Rust 1.70+）

### 2. 发布到 crates.io (Release)

**触发方式**：手动触发（GitHub UI）

**步骤**：
1. 访问：https://github.com/loyalpartner/searpc-rs/actions
2. 选择 "Release" workflow
3. 点击 "Run workflow"
4. 选择发布级别：
   - `patch`: 0.1.1 → 0.1.2 (bug 修复)
   - `minor`: 0.1.1 → 0.2.0 (新功能)
   - `major`: 0.1.1 → 1.0.0 (破坏性变更)

## 认证方式

### 方式 1: Trusted Publishing（推荐）

**优点**：
- ✅ 无需存储长期 token
- ✅ 自动管理临时凭证（30分钟过期）
- ✅ 更安全

**配置步骤**：

1. **首次发布**（必须手动完成一次）
   ```bash
   cargo login
   cargo release patch --workspace --execute
   ```

2. **在 crates.io 配置 Trusted Publisher**
   - 访问：https://crates.io/settings/tokens
   - 为每个 crate (searpc-macro, searpc, seaf-cli) 配置：
     - GitHub Repository: `loyalpartner/searpc-rs`
     - Workflow: `release.yml`
     - Environment: 留空

3. **后续发布**
   - 通过 GitHub Actions 自动发布

### 方式 2: API Token（备选）

**如果 Trusted Publishing 不可用**：

1. 获取 crates.io API token：
   - 访问：https://crates.io/settings/tokens
   - 创建新 token
   - 复制 token（只显示一次）

2. 添加到 GitHub Secrets：
   - 访问：https://github.com/loyalpartner/searpc-rs/settings/secrets/actions
   - 新建 secret：`CARGO_TOKEN`
   - 粘贴 token

3. Workflow 会自动使用这个 token

## 本地发布

仍然可以本地发布：

```bash
# 检查会发生什么
cargo release patch --workspace

# 执行发布
cargo release patch --workspace --execute
```

## 发布流程

```
1. 运行测试 → 2. 更新版本号 → 3. 发布到 crates.io → 4. 创建 git tag → 5. 推送到 GitHub
```

## 注意事项

- ⚠️ 每个版本只能发布一次（不可覆盖）
- ⚠️ 发布后不能删除（可以 yank）
- ✅ 确保所有测试通过再发布
- ✅ 遵循语义化版本规范

## 故障排查

### 问题：Trusted Publishing 失败

**解决方案**：
1. 检查 crates.io 配置是否正确
2. 确认 GitHub repo 名称匹配
3. 临时使用 API Token 方式

### 问题：依赖版本冲突

**解决方案**：
```bash
# 更新 Cargo.lock
cargo update
git add Cargo.lock
git commit -m "chore: update dependencies"
```

## 参考资料

- [cargo-release 文档](https://github.com/crate-ci/cargo-release)
- [Trusted Publishing RFC](https://rust-lang.github.io/rfcs/3691-trusted-publishing-cratesio.html)
- [crates.io 发布指南](https://doc.rust-lang.org/cargo/reference/publishing.html)
