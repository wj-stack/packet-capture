# Packet Capture - 网络数据包捕获工具

一个基于 Windows 平台的网络数据包捕获和分析工具，通过 DLL 注入和 API Hook 技术实时拦截和监控应用程序的网络通信。

## ✨ 功能特性

### 核心功能
- 🔍 **网络流量拦截**：通过 DLL 注入和 API Hook 技术拦截目标进程的网络通信
- 📦 **数据包捕获**：实时捕获 TCP、UDP、HTTP、HTTPS 等协议的数据包
- 📊 **数据解析与展示**：解析并可视化展示捕获的数据包内容
- 🔎 **过滤与搜索**：支持按协议、域名、关键词等条件过滤和搜索
- 📤 **导出功能**：支持将捕获的数据导出为多种格式（JSON、HAR、PCAP 等）
- 🎮 **游戏代理嗅探**：专门针对游戏网络通信的代理嗅探功能
- ⚙️ **数据包篡改**：支持配置篡改规则，实时修改或拦截数据包

### 支持的 Hook 类型
- `send` / `recv` - 基础 Socket 发送/接收
- `sendto` / `recvfrom` - UDP Socket 发送/接收
- `WSASend` / `WSARecv` - Windows Socket API 发送/接收
- `WSASendTo` / `WSARecvFrom` - Windows Socket API UDP 发送/接收