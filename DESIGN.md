# 抓包工具设计文档

## 1. 项目概述

### 1.1 项目简介
本项目旨在开发一个基于Windows平台的网络抓包工具，通过HOOK技术拦截和监控应用程序的网络通信，提供可视化的数据包分析和展示功能。

### 1.2 核心功能
- **网络流量拦截**：通过DLL注入和API HOOK技术拦截目标进程的网络通信
- **数据包捕获**：实时捕获HTTP/HTTPS、TCP、UDP等协议的数据包
- **数据解析与展示**：解析并可视化展示捕获的数据包内容
- **过滤与搜索**：支持按协议、域名、关键词等条件过滤和搜索
- **导出功能**：支持将捕获的数据导出为多种格式（JSON、HAR、PCAP等）

### 1.3 技术栈
- **前端框架**：React + TypeScript + Vite
- **桌面框架**：Tauri 2.0
- **后端语言**：Rust
- **HOOK技术**：Windows DLL注入 + API HOOK
- **网络库**：WinPcap/Npcap 或 Raw Socket

## 2. 系统架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                     前端层 (React)                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │ 主界面   │  │ 数据展示 │  │ 过滤器   │  │ 设置面板 ││
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘│
└─────────────────────────────────────────────────────────┘
                          ↕ IPC通信
┌─────────────────────────────────────────────────────────┐
│                  Tauri 后端层 (Rust)                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │ 命令处理 │  │ 事件管理 │  │ 数据管理 │  │ 配置管理 ││
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘│
└─────────────────────────────────────────────────────────┘
                          ↕ FFI调用
┌─────────────────────────────────────────────────────────┐
│                  HOOK核心层 (Rust DLL)                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │ DLL注入  │  │ API HOOK │  │ 数据捕获 │  │ 协议解析 ││
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘│
└─────────────────────────────────────────────────────────┘
                          ↕ 系统调用
┌─────────────────────────────────────────────────────────┐
│              Windows API (WinSock2, WinHTTP等)            │
└─────────────────────────────────────────────────────────┘
```

### 2.2 模块划分

#### 2.2.1 前端模块 (React)
- **主界面模块** (`src/components/MainPanel/`)
  - 进程列表展示
  - 抓包控制按钮
  - 状态指示器
  
- **数据展示模块** (`src/components/PacketList/`)
  - 数据包列表
  - 数据包详情面板
  - 实时更新机制
  
- **过滤器模块** (`src/components/FilterPanel/`)
  - 协议过滤
  - 域名过滤
  - 关键词搜索
  
- **设置模块** (`src/components/Settings/`)
  - 抓包配置
  - 界面主题
  - 导出设置

#### 2.2.2 Tauri后端模块 (Rust)
- **命令处理模块** (`src-tauri/src/commands/`)
  - `start_capture`: 开始抓包
  - `stop_capture`: 停止抓包
  - `get_packets`: 获取数据包列表
  - `filter_packets`: 过滤数据包
  - `export_packets`: 导出数据包
  
- **事件管理模块** (`src-tauri/src/events/`)
  - 数据包事件发射
  - 状态变更事件
  
- **数据管理模块** (`src-tauri/src/storage/`)
  - 数据包存储
  - 内存管理
  - 缓存策略

#### 2.2.3 HOOK核心模块 (Rust DLL)
- **注入模块** (`hook-dll/src/inject/`)
  - 进程枚举
  - DLL注入（CreateRemoteThread/SetWindowsHookEx）
  - 注入状态管理
  
- **HOOK模块** (`hook-dll/src/hook/`)
  - WinSock2 API HOOK (send/recv/WSASend/WSARecv)
  - WinHTTP API HOOK (WinHttpSendRequest/WinHttpReceiveResponse)
  - HTTP API HOOK (HttpSendRequest/InternetReadFile)
  - SSL/TLS HOOK (Schannel相关API)
  
- **数据捕获模块** (`hook-dll/src/capture/`)
  - 原始数据捕获
  - 数据缓冲
  - 数据发送到主进程
  
- **协议解析模块** (`hook-dll/src/parser/`)
  - HTTP协议解析
  - HTTPS数据提取（通过HOOK SSL API）
  - TCP/UDP协议解析

## 3. 技术实现方案

### 3.1 DLL注入方案

#### 3.1.1 注入方法选择
- **方法1：CreateRemoteThread + LoadLibrary**
  - 优点：兼容性好，实现简单
  - 缺点：可能被安全软件拦截
  
- **方法2：SetWindowsHookEx**
  - 优点：隐蔽性较好
  - 缺点：需要目标进程有窗口消息循环
  
- **方法3：Manual DLL Mapping**
  - 优点：绕过LoadLibrary检测
  - 缺点：实现复杂，需要手动处理重定位

**推荐方案**：优先使用CreateRemoteThread，失败时尝试SetWindowsHookEx

#### 3.1.2 注入流程
```
1. 枚举目标进程
2. 打开进程句柄 (OpenProcess)
3. 在目标进程中分配内存 (VirtualAllocEx)
4. 写入DLL路径 (WriteProcessMemory)
5. 创建远程线程执行LoadLibrary (CreateRemoteThread)
6. 等待DLL加载完成
7. 清理资源
```

### 3.2 API HOOK方案

#### 3.2.1 HOOK技术选择
- **方法1：Inline Hook (直接修改函数入口)**
  - 优点：性能好，通用性强
  - 缺点：需要处理多线程同步，可能被检测
  
- **方法2：IAT Hook (修改导入表)**
  - 优点：实现相对简单
  - 缺点：只能HOOK导入的函数
  
- **方法3：EAT Hook (修改导出表)**
  - 优点：可以HOOK模块导出函数
  - 缺点：适用范围有限

**推荐方案**：使用Inline Hook，通过修改函数前几个字节实现跳转

#### 3.2.2 需要HOOK的关键API

**WinSock2 API:**
- `send` / `WSASend` - 发送数据
- `recv` / `WSARecv` - 接收数据
- `connect` - 建立连接
- `closesocket` - 关闭连接

**WinHTTP API:**
- `WinHttpSendRequest` - 发送HTTP请求
- `WinHttpReceiveResponse` - 接收HTTP响应
- `WinHttpReadData` - 读取响应数据
- `WinHttpQueryDataAvailable` - 查询可用数据

**HTTP API:**
- `HttpSendRequest` - 发送HTTP请求
- `InternetReadFile` - 读取数据

**SSL/TLS API (Schannel):**
- `EncryptMessage` - 加密消息（用于捕获HTTPS请求）
- `DecryptMessage` - 解密消息（用于捕获HTTPS响应）

#### 3.2.3 HOOK实现示例（伪代码）
```rust
// HOOK函数结构
struct HookContext {
    original_func: *mut c_void,
    hook_func: *mut c_void,
    trampoline: Vec<u8>,
}

// 安装HOOK
fn install_hook(target_func: *mut c_void, hook_func: *mut c_void) -> Result<HookContext> {
    // 1. 保存原始函数前N个字节
    // 2. 构造跳转指令 (JMP hook_func)
    // 3. 修改目标函数入口
    // 4. 创建trampoline函数（原始函数+跳转回原地址）
}

// HOOK处理函数
#[no_mangle]
extern "C" fn hooked_send(socket: SOCKET, buf: *const u8, len: i32, flags: i32) -> i32 {
    // 1. 捕获原始数据
    capture_data(buf, len);
    
    // 2. 调用原始函数
    let original = get_original_send();
    original(socket, buf, len, flags)
}
```

### 3.3 数据流转方案

#### 3.3.1 数据流路径
```
目标进程 (HOOK DLL)
    ↓ 捕获数据
共享内存 / 命名管道 / Socket
    ↓ 传输数据
Tauri后端 (Rust)
    ↓ IPC通信
前端 (React)
```

#### 3.3.2 进程间通信方案

**方案1：命名管道 (Named Pipe)**
- 优点：Windows原生支持，性能好
- 缺点：需要处理连接管理

**方案2：共享内存 (Shared Memory)**
- 优点：性能最优
- 缺点：需要处理同步问题

**方案3：Socket通信**
- 优点：跨进程通信简单
- 缺点：性能略差

**推荐方案**：使用命名管道，配合事件对象实现同步

### 3.4 HTTPS解密方案

#### 3.4.1 方案选择
- **方案1：HOOK SSL/TLS API**
  - 在应用层HOOK Schannel的EncryptMessage/DecryptMessage
  - 优点：可以获取明文数据
  - 缺点：需要处理不同SSL库
  
- **方案2：中间人代理**
  - 设置本地代理，使用自签名证书
  - 优点：实现相对简单
  - 缺点：需要配置代理，可能被检测

**推荐方案**：优先使用SSL API HOOK，作为备选支持代理模式

### 3.5 可用Rust库推荐

本项目可以充分利用现有的Rust生态系统库来简化开发，以下是推荐的库及其用途：

#### 3.5.1 DLL注入相关库

**dll-syringe** (`crates.io/crates/dll-syringe`)
- **用途**：专业的DLL注入库，支持将DLL注入到目标进程
- **特点**：
  - 支持CreateRemoteThread注入方式
  - 提供DLL弹出（eject）功能
  - 支持32位和64位进程
  - API简洁易用
- **使用场景**：替代手动实现DLL注入逻辑

**dll-injector** (`crates.io/crates/dll-injector`)
- **用途**：另一个DLL注入实现
- **特点**：轻量级，提供基本的注入功能
- **备选方案**：如果dll-syringe不满足需求时的备选

#### 3.5.2 API HOOK相关库

本项目需要HOOK Windows API来拦截网络通信，以下是主要的HOOK库选择：

##### 3.5.2.1 detour (纯Rust实现)

**detour** (`crates.io/crates/detour`)
- **用途**：纯Rust实现的函数钩子库
- **优点**：
  - ✅ 纯Rust实现，无需外部依赖
  - ✅ 类型安全的HOOK接口
  - ✅ 支持静态和动态HOOK
  - ✅ 支持x86和x64架构
  - ✅ 开源免费，无许可限制
  - ✅ 易于集成到Rust项目
- **缺点**：
  - ❌ 功能相对基础，不支持复杂HOOK场景
  - ❌ 社区规模较小
  - ❌ 可能不如C/C++实现的库成熟
- **使用场景**：简单的API HOOK需求，优先考虑纯Rust方案时

##### 3.5.2.2 Microsoft Detours (通过detours-rs绑定)

**detours-rs** (`crates.io/crates/detours-rs`) - Microsoft Detours的Rust绑定

**Microsoft Detours** 是微软官方开发的API拦截库，功能强大且成熟。

**优点**：
- ✅ **功能全面**：支持复杂的API拦截和重定向场景
- ✅ **多架构支持**：支持x86、x64、ARM、ARM64架构
- ✅ **成熟稳定**：由微软开发和维护，在生产环境中广泛使用
- ✅ **文档完善**：官方文档详细，示例丰富
- ✅ **高级功能**：支持DLL注入、进程创建HOOK等高级功能
- ✅ **线程安全**：内置线程安全机制，适合多线程环境

**缺点**：
- ❌ **许可限制**：
  - Express版本（免费）**不支持x64架构**，仅支持x86
  - Professional版本支持x64，但需要购买商业许可证（价格昂贵）
  - 对于商业项目，许可成本可能较高
- ❌ **复杂性**：API相对复杂，学习曲线较陡
- ❌ **外部依赖**：需要链接Microsoft Detours的C++库
- ❌ **Rust绑定**：detours-rs可能不如原生C++ API完善

**适用场景**：
- 需要复杂HOOK功能的企业级项目
- 有预算购买Professional许可证的项目
- 需要ARM/ARM64架构支持的项目

##### 3.5.2.3 MinHook (通过minhook-rs绑定)

**minhook-rs** (`crates.io/crates/minhook`) - MinHook的Rust绑定

**MinHook** 是一个轻量级的x86/x64 API钩子库，专注于提供基本的HOOK功能。

**优点**：
- ✅ **开源免费**：完全开源，无许可限制，可自由使用
- ✅ **轻量级**：代码量小，体积小，性能优秀
- ✅ **易于使用**：API设计简洁，学习成本低
- ✅ **x64支持**：免费版本即支持x64架构
- ✅ **线程安全**：支持多线程环境下的HOOK
- ✅ **成熟稳定**：基于成熟的MinHook C库，经过大量项目验证
- ✅ **社区活跃**：GitHub上活跃，有良好的社区支持

**缺点**：
- ❌ **功能有限**：相比Detours，功能较为基础，不支持DLL注入等高级功能
- ❌ **架构支持**：仅支持x86和x64，不支持ARM/ARM64架构
- ❌ **文档相对较少**：相比Detours，文档和示例较少
- ❌ **外部依赖**：需要链接MinHook的C库

**适用场景**：
- 需要x64支持的免费方案
- 轻量级HOOK需求
- 预算有限的开源项目
- 只需要基本的函数HOOK功能

##### 3.5.2.4 对比总结

| 特性 | detour | Detours (detours-rs) | MinHook (minhook-rs) |
|------|--------|---------------------|---------------------|
| **实现方式** | 纯Rust | C++库 + Rust绑定 | C库 + Rust绑定 |
| **许可** | 开源免费 | Express免费(仅x86)，Pro需付费 | 开源免费 |
| **架构支持** | x86, x64 | x86, x64, ARM, ARM64 | x64 |
| **功能丰富度** | 基础 | 非常丰富 | 基础 |
| **易用性** | 简单 | 复杂 | 简单 |
| **稳定性** | 良好 | 优秀 | 优秀 |
| **性能** | 良好 | 优秀 | 优秀 |
| **文档** | 一般 | 完善 | 一般 |
| **社区支持** | 较小 | 大 | 中等 |

##### 3.5.2.5 推荐选择

**对于本项目（Windows抓包工具）的推荐**：

1. **首选：minhook-rs**
   - ✅ 免费且支持x64（本项目必需）
   - ✅ 功能足够满足HOOK WinSock2、WinHTTP、SSL API的需求
   - ✅ 轻量级，性能优秀
   - ✅ 成熟稳定，适合生产环境

2. **备选：detour**
   - ✅ 如果希望完全避免外部C库依赖
   - ✅ 如果只需要简单的HOOK功能
   - ⚠️ 需要验证其稳定性和功能完整性

3. **不推荐：detours-rs**
   - ❌ Express版本不支持x64（本项目必需）
   - ❌ Professional版本需要付费，成本高
   - ✅ 如果项目预算充足且需要高级功能，可以考虑

**最终建议**：使用 **minhook-rs**，它提供了免费、稳定、功能足够的HOOK能力，完全满足本项目的需求。

**技术选型总结**：

| 选择 | 理由 |
|------|------|
| **minhook-rs** | ✅ 免费且支持x64（必需）<br>✅ 成熟稳定，经过大量项目验证<br>✅ 功能足够满足HOOK需求<br>✅ 轻量级，性能优秀<br>✅ 开源，无许可限制 |
| ~~detours-rs~~ | ❌ Express版本不支持x64<br>❌ Professional版本需要付费<br>✅ 如果预算充足且需要高级功能可考虑 |
| ~~detour~~ | ✅ 纯Rust实现<br>⚠️ 功能相对基础<br>⚠️ 需要验证稳定性<br>✅ 如果希望避免C库依赖可考虑 |

#### 3.5.3 Windows API绑定

**windows** (`crates.io/crates/windows`)
- **用途**：微软官方提供的现代Windows API绑定
- **特点**：
  - 官方维护，更新及时
  - 类型安全，IDE支持好
  - 支持所有Windows API
  - 使用现代Rust特性
- **使用场景**：所有Windows API调用（推荐使用）

**winapi** (`crates.io/crates/winapi`)
- **用途**：传统的Windows API绑定
- **特点**：成熟稳定，但维护较少
- **备选方案**：如果windows crate不支持的API

#### 3.5.4 进程间通信库

**interprocess** (`crates.io/crates/interprocess`)
- **用途**：跨平台进程间通信库
- **特点**：
  - 支持命名管道（Windows）
  - 支持Unix域套接字（Linux/macOS）
  - 统一的API接口
  - 异步支持
- **使用场景**：DLL与Tauri后端之间的数据传输

**named-pipe** (`crates.io/crates/named-pipe`)
- **用途**：专门的Windows命名管道库
- **特点**：轻量级，专注于命名管道
- **备选方案**：如果只需要命名管道功能

#### 3.5.5 网络数据包处理库

**pnet** (`crates.io/crates/pnet`)
- **用途**：跨平台网络底层库
- **特点**：
  - 支持数据包发送和接收
  - 支持多种网络协议
  - 跨平台支持
- **使用场景**：如果需要底层网络操作（本项目主要通过HOOK，可能不需要）

**etherparse** (`crates.io/crates/etherparse`)
- **用途**：网络协议解析库
- **特点**：
  - 零拷贝解析
  - 支持以太网、IP、TCP、UDP等协议
  - 性能优秀
- **使用场景**：解析捕获的网络数据包

**rscap** (`crates.io/crates/rscap`)
- **用途**：网络数据包捕获、传输和构建
- **特点**：Rust原生实现，提供完整的捕获API
- **使用场景**：如果需要额外的数据包捕获能力

**rtshark** (`crates.io/crates/rtshark`)
- **用途**：TShark（Wireshark命令行工具）的Rust接口
- **特点**：
  - 强大的协议解析能力
  - 支持从实时捕获或文件读取
  - 可以解码各种协议
- **使用场景**：高级协议解析和数据分析

#### 3.5.6 HTTP/HTTPS处理库

**httparse** (`crates.io/crates/httparse`)
- **用途**：轻量级HTTP解析器
- **特点**：
  - 零分配解析
  - 性能优秀
  - 只解析，不验证
- **使用场景**：解析HTTP请求和响应（已包含在依赖中）

**http** (`crates.io/crates/http`)
- **用途**：HTTP类型和工具
- **特点**：提供HTTP相关的类型定义
- **使用场景**：HTTP数据结构的定义

**url** (`crates.io/crates/url`)
- **用途**：URL解析库
- **特点**：符合标准的URL解析
- **使用场景**：解析和操作URL（已包含在依赖中）

#### 3.5.7 异步运行时

**tokio** (`crates.io/crates/tokio`)
- **用途**：异步运行时
- **特点**：
  - 高性能异步I/O
  - 丰富的异步工具
  - 生态完善
- **使用场景**：异步处理数据包捕获和传输

#### 3.5.8 其他实用库

**serde** + **serde_json** (`crates.io/crates/serde`)
- **用途**：序列化和反序列化
- **特点**：Rust生态中最流行的序列化库
- **使用场景**：前后端数据交换、数据存储（已包含在依赖中）

**anyhow** (`crates.io/crates/anyhow`)
- **用途**：错误处理库
- **特点**：简化错误处理，提供上下文信息
- **使用场景**：统一错误处理

**thiserror** (`crates.io/crates/thiserror`)
- **用途**：自定义错误类型
- **特点**：简化错误类型定义
- **使用场景**：定义项目特定的错误类型

**log** + **env_logger** (`crates.io/crates/log`)
- **用途**：日志记录
- **特点**：标准化的日志接口
- **使用场景**：调试和日志记录

#### 3.5.9 库选择建议

**核心库（必须）**：
- `windows` - Windows API绑定
- `dll-syringe` - DLL注入
- `minhook` - API HOOK（推荐使用minhook-rs，免费且支持x64）
- `interprocess` - 进程间通信
- `serde` + `serde_json` - 序列化
- `httparse` - HTTP解析

**推荐库（强烈建议）**：
- `tokio` - 异步运行时
- `etherparse` - 协议解析
- `anyhow` - 错误处理
- `log` - 日志记录

**可选库（按需使用）**：
- `rtshark` - 高级协议解析（如果需要）
- `pnet` - 底层网络操作（如果需要）
- `detour` - 纯Rust HOOK库（如果希望避免C库依赖）
- `detours-rs` - Microsoft Detours绑定（如果需要高级功能且有预算）

## 4. 项目结构

### 4.1 目录结构
```
packet-capture/
├── src/                          # React前端代码
│   ├── components/              # React组件
│   │   ├── MainPanel/           # 主界面
│   │   ├── PacketList/          # 数据包列表
│   │   ├── PacketDetail/       # 数据包详情
│   │   ├── FilterPanel/         # 过滤器面板
│   │   └── Settings/            # 设置面板
│   ├── hooks/                   # React Hooks
│   ├── utils/                   # 工具函数
│   ├── types/                   # TypeScript类型定义
│   └── App.tsx                  # 主应用组件
│
├── src-tauri/                   # Tauri后端代码
│   ├── src/
│   │   ├── commands/            # Tauri命令
│   │   │   ├── capture.rs       # 抓包相关命令
│   │   │   ├── packet.rs        # 数据包相关命令
│   │   │   └── export.rs        # 导出相关命令
│   │   ├── events/              # 事件管理
│   │   ├── storage/             # 数据存储
│   │   ├── ipc/                 # IPC通信处理
│   │   └── lib.rs               # 库入口
│   └── Cargo.toml
│
├── hook-dll/                    # HOOK DLL项目
│   ├── src/
│   │   ├── inject/              # DLL注入相关
│   │   ├── hook/                # API HOOK实现
│   │   │   ├── winsock.rs       # WinSock2 HOOK
│   │   │   ├── winhttp.rs       # WinHTTP HOOK
│   │   │   └── ssl.rs           # SSL/TLS HOOK
│   │   ├── capture/             # 数据捕获
│   │   ├── parser/              # 协议解析
│   │   │   ├── http.rs          # HTTP解析
│   │   │   └── tcp.rs           # TCP解析
│   │   └── lib.rs               # DLL入口
│   └── Cargo.toml
│
├── shared/                      # 共享代码（前后端共用）
│   └── types.rs                 # 共享类型定义
│
└── docs/                        # 文档
    └── DESIGN.md                # 本文档
```

### 4.2 关键依赖

#### 4.2.1 Rust依赖 (Cargo.toml)

**Tauri后端依赖** (`src-tauri/Cargo.toml`):
```toml
[dependencies]
# Tauri核心
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Windows API（推荐使用windows crate）
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_System_Threading",
    "Win32_System_Memory",
    "Win32_System_LibraryLoader",
    "Win32_Networking_WinSock",
    "Win32_Networking_WinHttp",
    "Win32_Security",
    "Win32_System_Diagnostics_ToolHelp",
] }

# 进程间通信
interprocess = "2.0"

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 日志
log = "0.4"
env_logger = "0.11"

# 数据解析
httparse = "1.8"
url = "2.5"
```

**HOOK DLL依赖** (`hook-dll/Cargo.toml`):
```toml
[dependencies]
# DLL注入
dll-syringe = "0.9"

# API HOOK（推荐使用minhook，免费且支持x64）
minhook = "0.1"

# Windows API
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_System_Threading",
    "Win32_System_Memory",
    "Win32_System_LibraryLoader",
    "Win32_Networking_WinSock",
    "Win32_Networking_WinHttp",
    "Win32_Security",
] }

# 进程间通信
interprocess = "2.0"

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 协议解析
etherparse = "0.15"
httparse = "1.8"
url = "2.5"

# 错误处理
anyhow = "1.0"

# 日志
log = "0.4"
```

**共享库依赖** (`shared/Cargo.toml`):
```toml
[dependencies]
# 序列化（前后端共用类型）
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

#### 4.2.2 前端依赖 (package.json)
```json
{
  "dependencies": {
    "react": "^19.1.0",
    "react-dom": "^19.1.0",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-opener": "^2",
    "antd": "^5.0.0",  // UI组件库
    "@ant-design/icons": "^5.0.0",
    "dayjs": "^1.11.0"  // 时间处理
  }
}
```

## 5. 核心功能设计

### 5.1 进程选择与注入

#### 5.1.1 功能描述
- 列出系统中所有运行进程
- 支持按进程名、PID搜索
- 选择目标进程进行DLL注入
- 显示注入状态

#### 5.1.2 实现要点
- 使用`windows` crate的`Win32_System_Diagnostics_ToolHelp`模块枚举进程
- 或使用`dll-syringe`库提供的进程枚举功能
- 检查进程权限（需要管理员权限）
- 注入前验证目标进程架构（32位/64位）
- 使用`dll-syringe`库简化注入流程

### 5.2 数据包捕获

#### 5.2.1 功能描述
- 实时捕获目标进程的网络通信
- 支持HTTP/HTTPS、TCP、UDP协议
- 显示数据包的详细信息（时间、协议、大小、方向等）

#### 5.2.2 数据包结构
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Packet {
    pub id: u64,                    // 数据包ID
    pub timestamp: i64,             // 时间戳
    pub process_id: u32,            // 进程ID
    pub process_name: String,       // 进程名
    pub protocol: Protocol,         // 协议类型
    pub direction: Direction,       // 方向（发送/接收）
    pub src_addr: String,           // 源地址
    pub dst_addr: String,           // 目标地址
    pub size: usize,                // 数据大小
    pub headers: PacketHeaders,     // 协议头信息
    pub payload: Vec<u8>,           // 载荷数据
    pub parsed_data: Option<ParsedData>, // 解析后的数据
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Protocol {
    HTTP,
    HTTPS,
    TCP,
    UDP,
    Other(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Direction {
    Send,    // 发送
    Receive, // 接收
}
```

### 5.3 数据包解析

#### 5.3.1 HTTP协议解析
- 解析HTTP请求行/状态行
- 解析HTTP头部
- 解析HTTP Body
- 支持gzip/deflate解压

#### 5.3.2 HTTPS协议解析
- 通过HOOK SSL API获取明文
- 解析TLS握手信息
- 提取SNI（Server Name Indication）

#### 5.3.3 TCP/UDP协议解析
- 解析IP头
- 解析TCP/UDP头
- 显示端口信息

### 5.4 过滤与搜索

#### 5.4.1 过滤功能
- 按协议过滤（HTTP/HTTPS/TCP/UDP）
- 按域名过滤
- 按HTTP方法过滤（GET/POST等）
- 按状态码过滤
- 按数据大小过滤

#### 5.4.2 搜索功能
- 关键词搜索（支持URL、Header、Body）
- 正则表达式搜索
- 高亮显示匹配内容

### 5.5 数据导出

#### 5.5.1 导出格式
- **JSON格式**：结构化数据，便于程序处理
- **HAR格式**：HTTP Archive格式，兼容浏览器开发者工具
- **PCAP格式**：标准抓包格式，可用Wireshark打开
- **CSV格式**：表格数据，便于Excel分析

#### 5.5.2 导出内容
- 可导出单个数据包
- 可导出过滤后的数据包列表
- 可导出所有捕获的数据包

## 6. 安全与权限

### 6.1 权限要求
- **管理员权限**：DLL注入需要管理员权限
- **调试权限**：需要`SeDebugPrivilege`权限

### 6.2 安全考虑
- 注入前检查目标进程是否受保护（如系统进程）
- 提供白名单/黑名单机制
- 避免注入关键系统进程
- 提供明确的用户提示和确认

### 6.3 错误处理
- 注入失败时的错误提示
- HOOK失败时的降级方案
- 进程退出时的资源清理

## 7. 性能优化

### 7.1 数据捕获优化
- 使用环形缓冲区避免内存溢出
- 实现数据包采样机制（高流量时）
- 异步处理数据包解析

### 7.2 前端性能优化
- 虚拟滚动（处理大量数据包）
- 数据分页加载
- 防抖处理搜索和过滤

### 7.3 内存管理
- 限制内存中保存的数据包数量
- 自动清理旧数据包
- 提供手动清理功能

## 8. 开发计划

### 8.1 第一阶段：基础框架搭建（1-2周）
- [ ] 搭建Tauri + React项目结构
- [ ] 实现基础UI界面
- [ ] 实现进程列表功能
- [ ] 实现基础的IPC通信

### 8.2 第二阶段：DLL注入实现（2-3周）
- [ ] 创建HOOK DLL项目
- [ ] 实现DLL注入功能
- [ ] 实现基础的API HOOK（WinSock2）
- [ ] 实现数据捕获和传输

### 8.3 第三阶段：协议解析（2-3周）
- [ ] 实现HTTP协议解析
- [ ] 实现TCP/UDP协议解析
- [ ] 实现数据包展示功能
- [ ] 实现基础过滤功能

### 8.4 第四阶段：HTTPS支持（2-3周）
- [ ] 实现SSL/TLS API HOOK
- [ ] 实现HTTPS数据解密
- [ ] 优化HTTPS数据展示

### 8.5 第五阶段：高级功能（2-3周）
- [ ] 实现高级过滤和搜索
- [ ] 实现数据导出功能
- [ ] 性能优化
- [ ] 错误处理和稳定性改进

### 8.6 第六阶段：测试与优化（1-2周）
- [ ] 功能测试
- [ ] 性能测试
- [ ] 兼容性测试
- [ ] 文档完善

## 9. 技术难点与解决方案

### 9.1 难点1：64位进程注入
**问题**：64位进程只能注入64位DLL，32位进程只能注入32位DLL
**解决方案**：
- 编译两个版本的DLL（32位和64位）
- 检测目标进程架构
- 根据架构选择对应的DLL

### 9.2 难点2：多线程环境下的HOOK
**问题**：多线程环境下HOOK可能造成竞态条件
**解决方案**：
- 使用互斥锁保护HOOK安装过程
- 使用原子操作确保HOOK的原子性
- 实现线程安全的trampoline

### 9.3 难点3：HTTPS解密
**问题**：不同应用使用不同的SSL库，HOOK难度大
**解决方案**：
- 优先支持Schannel（Windows系统SSL库）
- 支持常见第三方SSL库（OpenSSL等）
- 提供代理模式作为备选方案

### 9.4 难点4：性能问题
**问题**：高频网络通信可能导致性能瓶颈
**解决方案**：
- 使用异步处理
- 实现数据采样
- 优化数据结构和算法

## 10. 测试计划

### 10.1 单元测试
- DLL注入功能测试
- API HOOK功能测试
- 协议解析测试
- 数据过滤测试

### 10.2 集成测试
- 端到端抓包流程测试
- 多进程同时抓包测试
- 长时间运行稳定性测试

### 10.3 兼容性测试
- 不同Windows版本测试（Win10/Win11）
- 不同架构测试（32位/64位）
- 不同浏览器测试（Chrome/Edge/Firefox）
- 不同应用程序测试

## 11. 参考资料

### 11.1 Windows API文档
- [Windows API Documentation](https://docs.microsoft.com/en-us/windows/win32/api/)
- [Process Injection Techniques](https://www.elastic.co/blog/ten-process-injection-techniques-technical-survey-common-and-trending-process)

### 11.2 HOOK技术
- [Microsoft Detours](https://github.com/Microsoft/Detours)
- [MinHook](https://github.com/TsudaKageyu/minhook)
- [detour-rs](https://crates.io/crates/detour) - Rust HOOK库
- [dll-syringe](https://crates.io/crates/dll-syringe) - Rust DLL注入库

### 11.3 网络协议
- [HTTP/1.1 Specification](https://tools.ietf.org/html/rfc7231)
- [TLS Specification](https://tools.ietf.org/html/rfc8446)

### 11.4 Tauri文档
- [Tauri Documentation](https://v2.tauri.app/start/)

---

## 附录：关键代码结构示例

### A.1 DLL入口点
```rust
// hook-dll/src/lib.rs
use windows::Win32::System::LibraryLoader::*;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "stdcall" fn DllMain(
    hinst_dll: HINSTANCE,
    fdw_reason: u32,
    _lpv_reserved: *mut std::ffi::c_void,
) -> bool {
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            // 初始化HOOK
            if let Err(e) = init_hooks() {
                eprintln!("Failed to initialize hooks: {}", e);
                return false;
            }
        }
        DLL_PROCESS_DETACH => {
            // 清理HOOK
            cleanup_hooks();
        }
        _ => {}
    }
    true
}
```

### A.2 DLL注入示例（使用dll-syringe）
```rust
// src-tauri/src/commands/capture.rs
use dll_syringe::{process::OwnedProcess, Syringe};

#[tauri::command]
pub async fn inject_dll(process_id: u32) -> Result<String, String> {
    // 打开目标进程
    let target_process = OwnedProcess::from_pid(process_id)
        .map_err(|e| format!("无法打开进程: {}", e))?;
    
    // 创建注入器
    let syringe = Syringe::for_process(target_process);
    
    // 获取DLL路径
    let dll_path = std::path::Path::new("hook_dll.dll");
    
    // 注入DLL
    let injected_payload = syringe.inject(dll_path)
        .map_err(|e| format!("注入失败: {}", e))?;
    
    Ok(format!("注入成功，PID: {}", injected_payload.process_id()))
}
```

### A.3 Tauri命令示例
```rust
// src-tauri/src/commands/capture.rs
use dll_syringe::{process::OwnedProcess, Syringe};

#[tauri::command]
pub async fn start_capture(process_id: u32) -> Result<String, String> {
    // 执行DLL注入
    inject_dll(process_id).await
}

#[tauri::command]
pub async fn stop_capture(process_id: u32) -> Result<String, String> {
    // 停止抓包（弹出DLL）
    let target_process = OwnedProcess::from_pid(process_id)
        .map_err(|e| format!("无法打开进程: {}", e))?;
    
    let syringe = Syringe::for_process(target_process);
    
    // 弹出DLL
    syringe.eject()
        .map_err(|e| format!("弹出失败: {}", e))?;
    
    Ok("停止成功".to_string())
}
```

### A.4 API HOOK示例（使用minhook）

**使用minhook-rs进行API HOOK的示例**：

```rust
// hook-dll/src/hook/winsock.rs
use minhook::{Hook, MinHook};
use windows::Win32::Networking::WinSock::*;
use std::sync::OnceLock;

// 定义HOOK函数类型
type SendFn = unsafe extern "system" fn(SOCKET, *const u8, i32, i32) -> i32;

// 全局HOOK实例
static SEND_HOOK: OnceLock<Hook<SendFn>> = OnceLock::new();

// 原始函数指针
static mut ORIGINAL_SEND: Option<SendFn> = None;

// HOOK处理函数
unsafe extern "system" fn hooked_send(
    socket: SOCKET,
    buf: *const u8,
    len: i32,
    flags: i32,
) -> i32 {
    // 捕获数据
    if !buf.is_null() && len > 0 {
        let data = std::slice::from_raw_parts(buf, len as usize);
        capture_data(data);
    }
    
    // 调用原始函数
    if let Some(original) = ORIGINAL_SEND {
        original(socket, buf, len, flags)
    } else {
        -1 // 错误处理
    }
}

// 安装HOOK
pub fn install_send_hook() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // 获取原始函数地址
        let ws2_32 = windows::Win32::System::LibraryLoader::LoadLibraryA(
            windows::core::s!("ws2_32.dll")
        )?;
        
        let send_addr = windows::Win32::System::LibraryLoader::GetProcAddress(
            ws2_32,
            windows::core::s!("send")
        ).ok_or("无法找到send函数")?;
        
        let original_fn: SendFn = std::mem::transmute(send_addr);
        ORIGINAL_SEND = Some(original_fn);
        
        // 创建HOOK
        let hook = MinHook::create_hook(
            original_fn,
            hooked_send,
        )?;
        
        // 启用HOOK
        hook.enable()?;
        
        // 保存HOOK实例
        SEND_HOOK.set(hook).map_err(|_| "HOOK已存在")?;
    }
    
    Ok(())
}

// 卸载HOOK
pub fn uninstall_send_hook() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(hook) = SEND_HOOK.get() {
        hook.disable()?;
    }
    Ok(())
}
```

**注意**：以上代码为示例，实际使用时需要根据`minhook` crate的具体API进行调整。minhook-rs的API可能因版本而异，请参考最新文档。

**备选方案：使用detour（纯Rust）**：
如果希望避免C库依赖，也可以使用纯Rust的`detour`库：

```rust
// hook-dll/src/hook/winsock.rs (使用detour)
use detour::static_detour;
use windows::Win32::Networking::WinSock::*;

// 定义HOOK函数类型
type SendFn = unsafe extern "system" fn(SOCKET, *const u8, i32, i32) -> i32;

// 创建静态HOOK
static_detour! {
    static SendHook: unsafe extern "system" fn(SOCKET, *const u8, i32, i32) -> i32;
}

// HOOK处理函数
unsafe extern "system" fn hooked_send(
    socket: SOCKET,
    buf: *const u8,
    len: i32,
    flags: i32,
) -> i32 {
    // 捕获数据
    if !buf.is_null() && len > 0 {
        let data = std::slice::from_raw_parts(buf, len as usize);
        capture_data(data);
    }
    
    // 调用原始函数
    SendHook.call(socket, buf, len, flags)
}

// 安装HOOK
pub fn install_send_hook() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // 获取原始函数地址
        let ws2_32 = windows::Win32::System::LibraryLoader::LoadLibraryA(
            windows::core::s!("ws2_32.dll")
        )?;
        
        let send_addr = windows::Win32::System::LibraryLoader::GetProcAddress(
            ws2_32,
            windows::core::s!("send")
        ).ok_or("无法找到send函数")?;
        
        // 安装HOOK
        SendHook.initialize(
            std::mem::transmute(send_addr),
            hooked_send,
        )?.enable()?;
    }
    
    Ok(())
}
```

### A.3 React组件示例
```typescript
// src/components/MainPanel/MainPanel.tsx
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export function MainPanel() {
  const [packets, setPackets] = useState<Packet[]>([]);
  
  useEffect(() => {
    // 监听数据包事件
    const unlisten = listen<Packet>('packet-captured', (event) => {
      setPackets(prev => [...prev, event.payload]);
    });
    
    return () => {
      unlisten.then(fn => fn());
    };
  }, []);
  
  const handleStartCapture = async (pid: number) => {
    await invoke('start_capture', { processId: pid });
  };
  
  // ...
}
```

---

**文档版本**：v1.0  
**最后更新**：2024年  
**维护者**：开发团队

