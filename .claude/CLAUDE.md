# searpc-rs 项目状态

## 项目概述

Rust 实现的 Searpc RPC 协议，完全兼容 libsearpc C 和 Python 实现。用于与 Seafile 守护进程通信。

## 已完成功能

### Phase 1: 核心协议 ✅

- [x] JSON 序列化/反序列化（protocol.rs）
- [x] 类型系统（types.rs: Arg enum + IntoArg trait）
- [x] RPC 客户端（client.rs: 6 种 RPC 类型）
- [x] TCP Transport（16-bit big-endian header，用于 demo）
- [x] Unix Socket Transport（32-bit native-endian header，Seafile 生产环境）
- [x] 错误处理（error.rs: thiserror）
- [x] 与 libsearpc C demo server 完全兼容

### Phase 2: 类型安全宏 ✅

- [x] `#[rpc]` 过程宏（searpc-macro/）
- [x] Trait-based 客户端生成
- [x] `prefix` 支持（DRY 原则）
- [x] 自动类型转换：
  - `i32` → `bool`（0 = false, 非 0 = true）
  - `null` → `None` for `Option<T>`
  - `null` → `[]` for `Vec<T>`
- [x] 自动反序列化（JSON → 自定义类型）
- [x] 完全类型安全（编译时检查）

### Phase 3: Async 支持 ✅

- [x] Async RPC 客户端（async_client.rs）
- [x] Async TCP Transport（async_tcp_transport.rs）
- [x] Tokio 集成
- [x] Feature flags（`async`，默认启用）

### Phase 4: seaf-cli 工具 ✅

- [x] 命令行客户端（seaf-cli/）
- [x] 支持的命令：
  - `list` - 列出仓库
  - `status` - 显示同步状态
  - `config` - 配置管理
  - `stop` - 停止守护进程
- [x] 完整的 Seafile RPC 绑定
- [x] 零包装模式（直接在 `SearpcClient<T>` 上实现 trait）

## 架构设计

### 数据结构优先

```rust
// C: strcmp(type, "int") - 运行时字符串比较
// Rust: enum - 编译时类型检查
pub enum Arg {
    Int(i32),
    Int64(i64),
    String(String),
    Json(Value),
    Null,
}
```

### 消除特殊情况

```rust
// 坏代码：需要特殊处理
if result.is_null() {
    return Ok(Vec::new());
}

// 好代码：宏自动处理
#[rpc]
fn get_repo_list(&mut self) -> Result<Vec<Repo>>;
// 自动处理 null → []
```

### 传输层协议差异

**关键发现**：libsearpc 有两种不同的包协议！

1. **TCP Demo Protocol**（16-bit）：
   ```
   [u16 len (big-endian)][JSON]
   ```

2. **Unix Socket Protocol**（32-bit + wrapper）：
   ```
   [u32 len (native-endian)][Wrapped JSON]
   ```

   包装格式：
   ```json
   {
     "service": "seafile-rpcserver",
     "request": "[\"func\", arg1, ...]"  // JSON 字符串！
   }
   ```

**重要**：`request` 字段是 JSON 字符串，不是 JSON 数组！这是调试时发现的关键 bug。

### 错误处理

```rust
// searpc 库：使用 searpc::Result
pub type Result<T> = std::result::Result<T, SearpcError>;

pub enum SearpcError {
    RpcError { code: i32, message: String },
    TransportError(String),
    JsonError(#[from] serde_json::Error),
    InvalidResponse(String),
    TypeError(String),
    IoError(#[from] std::io::Error),
    EnvVarError(#[from] std::env::VarError),
}

// seaf-cli：使用 anyhow::Result（应用层可以）
```

## 宏实现细节

### 返回类型匹配

```rust
fn match_return_type(ty: &Type) -> Result<(CallMethod, Deserialize)> {
    if is_type(ty, "String") { ... }
    if is_type(ty, "i32") { ... }
    if is_type(ty, "bool") {
        // 自动转换：call_int() → bool
        return Ok((quote!(call_int), quote!(Ok(result != 0))));
    }
    if is_type(ty, "Vec") {
        // 自动反序列化 Vec<Value> → Vec<T>
        return Ok((quote!(call_objlist), deserialize_vec));
    }
    if is_type(ty, "Option") {
        // 自动处理 null → None
        return Ok((quote!(call_object), deserialize_option));
    }
}
```

### 代码生成

```rust
// 用户代码：
#[rpc(prefix = "seafile")]
trait SeafileRpc {
    fn is_auto_sync_enabled(&mut self) -> Result<bool>;
}

// 生成的代码：
impl<T: Transport> SeafileRpc for SearpcClient<T> {
    fn is_auto_sync_enabled(&mut self) -> Result<bool> {
        let args = vec![];
        let result = self.call_int("seafile_is_auto_sync_enabled", args)?;
        Ok(result != 0)  // 自动转换
    }
}
```

## 文件结构

```
searpc-rs/
├── searpc/                     # 核心 RPC 库
│   ├── src/
│   │   ├── lib.rs             # Re-exports
│   │   ├── protocol.rs        # RpcRequest/RpcResponse
│   │   ├── types.rs           # Arg + IntoArg
│   │   ├── client.rs          # SearpcClient (sync)
│   │   ├── error.rs           # SearpcError + Result
│   │   ├── transport.rs       # Transport trait
│   │   ├── tcp_transport.rs   # 16-bit header
│   │   ├── unix_transport.rs  # 32-bit header + wrapper
│   │   ├── async_client.rs    # AsyncSearpcClient
│   │   └── async_*.rs         # Async transports
│   └── examples/              # 示例代码
├── searpc-macro/               # 过程宏
│   └── src/lib.rs             # #[rpc] 实现（~430 行）
└── seaf-cli/                   # Seafile CLI
    ├── src/
    │   ├── main.rs            # CLI 入口
    │   └── rpc_client.rs      # Seafile RPC trait
    └── Cargo.toml
```

## 测试覆盖

- ✅ 单元测试（protocol, types, error）
- ✅ 集成测试（与 libsearpc demo server）
- ✅ 文档测试（lib.rs 中的示例）
- ✅ 实际使用（seaf-cli 与 Seafile 守护进程通信）

## 性能指标

- **代码量**：~1500 行 vs C 的 ~2000 行（-25%）
- **编译警告**：0
- **unsafe 代码块**：0
- **依赖项**：最小化（serde, serde_json, thiserror, tokio）

## 已知问题

无严重问题。

## 未来改进

- [ ] 连接池
- [ ] 超时和重试机制
- [ ] Server 实现（如果需要）
- [ ] 性能基准测试

## 设计哲学

遵循 Linus Torvalds 的 "good taste" 原则：

1. **数据结构优先**：用类型系统替代字符串比较
2. **消除特殊情况**：统一的代码路径
3. **内存安全**：零 unsafe，编译器验证
4. **实用主义**：解决实际问题，不过度设计

## Git 仓库

https://github.com/loyalpartner/searpc-rs

## 最后更新

2025-10-22 - Phase 4 完成
- 添加 seaf-cli
- 宏支持 bool 自动转换
- 简化 API（移除包装模式）
- 更新 README
