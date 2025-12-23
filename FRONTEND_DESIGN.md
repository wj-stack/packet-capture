# 前端设计文档

## 1. 设计概述

### 1.1 设计目标
- **易用性**：界面简洁直观，操作流程清晰
- **专业性**：提供专业的数据包分析功能
- **性能**：支持大量数据包的流畅展示和实时更新
- **可扩展性**：组件化设计，便于功能扩展

### 1.2 设计原则
- **信息层次清晰**：重要信息突出，次要信息可折叠
- **实时反馈**：操作即时反馈，状态变化明显
- **一致性**：统一的视觉风格和交互模式
- **可访问性**：支持键盘导航，符合无障碍标准

### 1.3 技术栈
- **框架**：React 19.1.0 + TypeScript
- **UI组件库**：Ant Design 5.0.0
- **状态管理**：React Hooks (useState, useContext, useReducer)
- **样式方案**：CSS Modules + Ant Design主题定制
- **构建工具**：Vite
- **桌面框架**：Tauri 2.0

## 2. 整体布局设计

### 2.1 主布局结构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  标题栏 (Title Bar)                                                         │
│  [应用图标] 抓包工具 V1.0                    [最小化][最大化][关闭]        │
├─────────────────────────────────────────────────────────────────────────────┤
│  主内容区 (Main Content) - 左右分栏布局                                     │
│  ┌──────────────────────────────────────────┬────────────────────────────┐ │
│  │  左侧主区域 (Left Panel)                  │  右侧面板 (Right Panel)    │ │
│  │                                          │                            │ │
│  │  ┌────────────────────────────────────┐ │  ┌──────────────────────┐  │ │
│  │  │  封包数据表格 (Packet List)         │ │  │ 封包拦截/过滤面板    │  │ │
│  │  │  序号|IP地址|函数|包长|套接字|数据  │ │  │ ┌──────────────────┐ │  │ │
│  │  │  362|124.238.245.112:443|Recv|...  │ │  │ │封包特征表格       │ │  │ │
│  │  │  363|39.107.227.119:443|Recv|...   │ │  │ │ID|类型|封包特征   │ │  │ │
│  │  │  ...                                │ │  │ └──────────────────┘ │  │ │
│  │  └────────────────────────────────────┘ │ │  │ 待过滤特征: [输入]  │  │ │
│  │                                          │ │  │ 替换为特征: [输入]  │  │ │
│  │  ┌────────────────────────────────────┐ │ │  │ [加入特征] [类型选择]│  │ │
│  │  │  抓包控制 (Capture Controls)         │ │ │  │ ☑仅记录 ☑过滤 ☑替换│  │ │
│  │  │  线程: [下拉] [开始抓包][结束][清空] │ │ │  └──────────────────────┘  │ │
│  │  │  ☑过滤包长 最短:[输入] 最长:[输入] │ │ │                            │ │
│  │  └────────────────────────────────────┘ │ │  ┌──────────────────────┐  │ │
│  │                                          │ │  │ 拦截状态显示区域    │  │ │
│  │  ┌────────────────────────────────────┐ │ │  │ (日志和状态信息)     │  │ │
│  │  │  封包函数过滤 (Function Filter)     │ │ │  └──────────────────────┘  │ │
│  │  │  ☑Send ☑Recv ☑SendTo ☑RecvFrom    │ │ │                            │ │
│  │  │  ☑WSASend ☑WSARecv ☑WSASendTo... │ │ │                            │ │
│  │  │  ☑全选                              │ │ │                            │ │
│  │  └────────────────────────────────────┘ │ │                            │ │
│  │                                          │ │                            │ │
│  │  ┌────────────────────────────────────┐ │ │                            │ │
│  │  │  封包预览 (Packet Preview)          │ │ │                            │ │
│  │  │  ┌──────────────────────────────┐ │ │ │                            │ │
│  │  │  │                              │ │ │ │                            │ │
│  │  │  │  封包内容显示区域             │ │ │ │                            │ │
│  │  │  │                              │ │ │ │                            │ │
│  │  │  └──────────────────────────────┘ │ │ │                            │ │
│  │  │  解密为=> [UTF-8▼] ☑实时预览      │ │ │                            │ │
│  │  └────────────────────────────────────┘ │ │                            │ │
│  └──────────────────────────────────────────┴────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│  状态栏 (Status Bar)                                                        │
│  已捕获: 1234 | 过滤后: 567 | 内存使用: 45MB | 状态: 运行中                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 布局尺寸规范

- **左侧主区域**：占据约70%宽度，包含数据包列表、控制面板、预览区域
- **右侧面板**：占据约30%宽度，包含拦截/过滤/修改功能
- **封包数据表格**：占据左侧区域的上半部分（约60%高度）
- **抓包控制区域**：高度约80px
- **封包函数过滤**：高度约100px
- **封包预览区域**：占据左侧区域的下半部分（约40%高度）
- **状态栏高度**：28px
- **最小窗口尺寸**：1200px × 700px

### 2.3 响应式设计

- **小屏幕（< 1200px）**：过滤器栏可折叠，减少列数显示
- **中等屏幕（1200px - 1600px）**：标准布局
- **大屏幕（> 1600px）**：可显示更多列信息，详情面板可并排显示

## 3. 组件详细设计

### 3.1 封包数据列表 (PacketList)

#### 3.1.1 功能描述
显示捕获的数据包列表，包含序号、IP地址、封包函数、包长、套接字、封包数据等信息。

#### 3.1.2 表格列定义
| 列名 | 宽度 | 说明 |
|------|------|------|
| 序号 | 80px | 数据包序号（从1开始） |
| IP地址 | 180px | 目标IP地址和端口 |
| 封包函数 | 100px | Send/Recv/WSASend等 |
| 包长 | 100px | 数据包大小（字节） |
| 套接字 | 100px | Socket句柄 |
| 封包数据 | 300px | 十六进制数据（可省略显示） |

#### 3.1.3 交互设计
- 点击行选择数据包，选中后会在封包预览区域显示
- 支持虚拟滚动，处理大量数据包
- 行样式区分发送（浅蓝）和接收（浅绿）

### 3.2 抓包控制面板 (CaptureControls)

#### 3.2.1 功能描述
提供抓包控制功能，包括线程选择、开始/结束/清空操作、包长过滤等。

#### 3.2.2 组件结构
```typescript
interface CaptureControlsProps {
  threadCount: number;
  onThreadCountChange: (count: number) => void;
  onStartCapture: () => void;
  onStopCapture: () => void;
  onClearData: () => void;
  filterBySize: boolean;
  minSize?: number;
  maxSize?: number;
}
```

#### 3.2.3 UI设计
- **线程选择**：下拉选择框，范围1-50
- **操作按钮**：开始抓包（Primary）、结束抓包、清空列表
- **包长过滤**：复选框 + 最短/最长输入框

### 3.3 封包函数过滤 (PacketFunctionFilter)

#### 3.3.1 功能描述
通过复选框过滤不同类型的封包函数（Send、Recv、WSASend等）。

#### 3.3.2 支持的封包函数
- Send
- Recv
- SendTo
- RecvFrom
- WSASend
- WSARecv
- WSASendTo
- WSARecvFrom

#### 3.3.3 交互设计
- 全选/取消全选功能
- 实时过滤数据包列表
- 支持多选

### 3.4 封包预览 (PacketPreview)

#### 3.4.1 功能描述
显示选中数据包的详细内容，支持多种解码格式。

#### 3.4.2 功能特性
- **解码格式**：UTF-8、GBK、ASCII、Hex
- **实时预览**：选中数据包后自动更新
- **文本区域**：只读，显示解码后的内容

### 3.5 封包拦截面板 (PacketInterceptPanel)

#### 3.5.1 功能描述
管理封包拦截、过滤、替换规则，显示拦截状态日志。

#### 3.5.2 组件结构
```typescript
interface PacketFeatureRule {
  id: string;
  type: PacketFunction;
  feature: string;        // 待过滤特征
  replacement?: string;   // 替换为特征
  onlyRecord: boolean;    // 仅记录次数
  filter: boolean;        // 过滤
  replace: boolean;       // 替换
  matchCount?: number;    // 匹配次数
}
```

#### 3.5.3 UI设计
- **特征规则表格**：显示所有规则（ID、类型、特征、替换、匹配次数）
- **特征输入**：待过滤特征、替换为特征
- **规则管理**：加入特征按钮、类型选择、选项复选框
- **拦截日志**：显示拦截状态和操作日志

### 3.6 顶部工具栏 (TopToolbar) - 已弃用

#### 3.1.1 功能描述
提供主要的操作按钮和快捷功能入口。注意：DLL注入功能暂时使用外部工具完成，本应用专注于数据包捕获、分析和操作。

#### 3.1.2 组件结构
```typescript
interface TopToolbarProps {
  onStartCapture: () => void;
  onStopCapture: () => void;
  onClearData: () => void;
  onTestSend: () => void;  // 测试发包
  onExport: () => void;
  onOpenSettings: () => void;
  captureStatus: 'idle' | 'capturing' | 'stopping';
}
```

#### 3.1.3 UI设计
- **按钮布局**：水平排列，图标+文字
- **按钮样式**：
  - 主要操作（开始/停止）：Primary按钮，带状态指示
  - 次要操作：Default按钮
  - 测试发包：Success按钮（绿色）
  - 危险操作（清空）：Danger按钮
- **状态指示**：
  - 空闲：灰色
  - 抓包中：绿色，带动画效果
  - 停止中：黄色
- **按钮顺序**：开始抓包 | 停止抓包 | 清空数据 | 测试发包 | 导出 | 设置

#### 3.1.4 交互设计
- 开始抓包：开始监听已注入DLL的进程数据包（无需选择进程，由DLL自动上报）
- 停止抓包：停止捕获数据包
- 测试发包：打开测试发包对话框
- 清空数据：清空当前捕获的所有数据包（需确认）

### 3.2 测试发包面板 (TestSendPanel)

#### 3.2.1 功能描述
提供测试发包功能，允许用户手动构造和发送TCP/UDP数据包进行测试。

#### 3.2.2 组件结构
```typescript
interface TestSendPanelProps {
  visible: boolean;
  onClose: () => void;
  onSend: (packet: TestPacket) => Promise<void>;
}

interface TestPacket {
  protocol: 'TCP' | 'UDP';
  dstAddr: string;  // 目标地址 IP:Port
  srcAddr?: string; // 源地址 IP:Port（可选）
  data: string | Uint8Array;  // 数据内容（十六进制或文本）
  targetProcessId?: number;  // 可选：指定目标进程
  delay?: number;  // 延迟发送（毫秒）
  tcpFlags?: number;  // TCP标志位（仅TCP）
}
```

#### 3.2.3 UI设计
- **布局**：模态对话框或侧边抽屉
- **表单结构**：
  - 协议选择：TCP/UDP
  - 目标地址：IP:Port输入框
  - 源地址：IP:Port输入框（可选）
  - 数据内容：十六进制或文本输入
  - 目标进程选择（可选）
  - 延迟发送（毫秒）
- **操作按钮**：
  - 发送：立即发送
  - 保存模板：保存为模板供后续使用
  - 从数据包导入：从选中的数据包创建
  - 取消：关闭对话框

#### 3.2.4 交互设计
- 打开方式：通过菜单或快捷键打开
- 实时验证：输入时实时验证IP地址格式等
- 模板功能：支持保存常用数据包模板
- 历史记录：显示最近发送的测试包
- 发送确认：发送前显示确认对话框（可关闭）

### 3.3 数据包替换面板 (PacketReplacePanel) - 已整合到封包拦截面板

注意：数据包替换功能已整合到封包拦截面板（PacketInterceptPanel）中，通过特征规则实现。

### 3.3 过滤器栏 (FilterBar)

#### 3.3.1 功能描述
提供多种过滤和搜索条件，实时过滤数据包列表。

#### 3.3.2 组件结构
```typescript
interface FilterBarProps {
  filters: FilterConditions;
  onFiltersChange: (filters: FilterConditions) => void;
  onClearFilters: () => void;
}

interface FilterConditions {
  protocols: Protocol[];        // 协议过滤
  direction?: 'send' | 'receive' | 'all';  // 方向过滤
  domain?: string;              // 域名过滤
  method?: string;             // HTTP方法过滤
  statusCode?: number;         // 状态码过滤
  minSize?: number;            // 最小大小
  maxSize?: number;            // 最大大小
  keyword?: string;            // 关键词搜索
  useRegex?: boolean;          // 是否使用正则
  searchIn: ('url' | 'header' | 'body')[];  // 搜索范围
}
```

#### 3.3.3 UI设计
- **布局**：已整合到各个专门的过滤组件中
- **过滤功能分布**：
  - 封包函数过滤：在封包函数过滤组件中
  - 包长过滤：在抓包控制面板中
  - 协议过滤：通过封包函数类型间接实现
  - 关键词搜索：可在封包预览中实现

#### 3.3.4 交互设计
- 实时过滤：输入后自动应用（防抖500ms）
- 过滤器状态：显示当前激活的过滤器数量
- 过滤器标签：可点击删除单个过滤器

### 3.4 数据包列表 (PacketList)

#### 3.4.1 功能描述
以表格形式展示捕获的数据包，支持虚拟滚动和排序。

#### 3.4.2 组件结构
```typescript
interface PacketListProps {
  packets: Packet[];
  selectedPacketId?: number;
  onSelectPacket: (packet: Packet) => void;
  onEditPacket: (packet: Packet) => void;
  onReplayPacket: (packet: Packet) => void;
  onReplacePacket: (packet: Packet) => void;
  filters: FilterConditions;
  loading?: boolean;
}

interface Packet {
  id: number;
  timestamp: number;
  processId: number;
  processName: string;
  protocol: Protocol;
  direction: 'send' | 'receive';
  srcAddr: string;
  dstAddr: string;
  size: number;
  socket?: number;
  packetFunction?: PacketFunction;
  packetData?: string;
}
```

#### 3.4.3 UI设计
- **表格列**：
  | 列名 | 宽度 | 说明 |
  |------|------|------|
  | 序号 | 80px | 数据包序号（从1开始） |
  | IP地址 | 180px | 目标IP地址和端口 |
  | 封包函数 | 100px | Send/Recv/WSASend等 |
  | 包长 | 100px | 数据包大小（字节） |
  | 套接字 | 100px | Socket句柄 |
  | 封包数据 | 300px | 十六进制数据（可省略显示） |

- **行样式**：
  - 发送数据包：浅蓝色背景
  - 接收数据包：浅绿色背景
  - 选中行：深蓝色边框和背景

#### 3.4.4 交互设计
- **虚拟滚动**：使用 `react-window` 或 `react-virtualized` 实现
- **排序**：点击列头排序，支持多列排序
- **选择**：点击行选择，Ctrl+点击多选，Shift+点击范围选择
- **操作按钮**（每行）：
  - **编辑**：打开编辑对话框，修改数据包内容
  - **重放**：重新发送该数据包（仅发送方向）
  - **替换**：设置替换规则，替换后续相同的数据包
- **右键菜单**：
  - 查看详情
  - 编辑数据包
  - 重放数据包
  - 设置替换规则
  - 复制URL
  - 复制请求头
  - 复制响应体
  - 导出选中
  - 在新窗口打开URL

#### 3.4.5 性能优化
- 虚拟滚动：只渲染可见区域的数据包
- 分页加载：初始加载1000条，滚动时动态加载
- 防抖更新：批量更新数据包，避免频繁重渲染

### 3.5 数据包详情面板 (PacketDetailPanel)

#### 3.5.1 功能描述
显示选中数据包的详细信息，支持多标签页展示。

#### 3.5.2 组件结构
```typescript
interface PacketDetailPanelProps {
  packet: Packet | null;
  onClose: () => void;
  onEdit: (packet: Packet) => void;
  onReplay: (packet: Packet) => void;
  onReplace: (packet: Packet) => void;
}

interface PacketDetail extends Packet {
  headers: Record<string, string>;
  requestBody?: string;
  responseBody?: string;
  rawData?: string;
  parsedData?: ParsedData;
}
```

#### 3.5.3 UI设计
- **预览区域布局**：左右分栏，中间控制区域
  - **左侧文本区域**：显示原始封包数据（十六进制格式）
    - 选择数据包后自动填充
    - 未选择数据包时可手动输入
    - 只读模式（当有选中数据包时）
  - **中间控制区域**：
    - "解密为=>" 标签
    - 解码格式下拉菜单（UTF-8、GBK、ASCII、Hex）
    - "实时预览" 复选框
  - **右侧文本区域**：显示解密后的数据
    - 根据左侧数据和选择的解码格式自动更新
    - 只读模式

- **布局结构**：
  ```
  ┌─────────────────────────────────────────────────────┐
  │  封包预览                                            │
  ├─────────────────────────────────────────────────────┤
  │  ┌──────────────┐  ┌──────────┐  ┌──────────────┐ │
  │  │              │  │ 解密为=>  │  │              │ │
  │  │  原始数据    │  │ [UTF-8▼] │  │  解密后数据  │ │
  │  │  (十六进制)  │  │          │  │              │ │
  │  │              │  │ ☑实时预览│  │              │ │
  │  │              │  │          │  │              │ │
  │  └──────────────┘  └──────────┘  └──────────────┘ │
  └─────────────────────────────────────────────────────┘
  ```

#### 3.5.4 交互设计
- **数据加载**：选择数据包后，左侧自动填充原始十六进制数据
- **手动输入**：未选择数据包时，可在左侧手动输入十六进制数据
- **解码格式切换**：切换下拉菜单中的格式，右侧立即更新解密结果
- **实时预览**：
  - 启用后：数据包变化时自动更新解密内容
  - 禁用后：需要手动切换格式或重新选择数据包才会更新
- **内容操作**：
  - 复制：右键菜单或快捷键复制内容
  - 搜索：Ctrl+F 搜索内容（浏览器原生支持）

### 3.6 设置面板 (SettingsPanel)

#### 3.6.1 功能描述
应用设置和配置管理。

#### 3.6.2 组件结构
```typescript
interface SettingsPanelProps {
  visible: boolean;
  onClose: () => void;
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

interface AppSettings {
  capture: {
    maxPackets: number;           // 最大数据包数量
    autoCleanup: boolean;         // 自动清理
    cleanupInterval: number;      // 清理间隔（分钟）
    bufferSize: number;           // 缓冲区大小
  };
  display: {
    theme: 'light' | 'dark' | 'auto';
    fontSize: number;
    showTimestamp: 'absolute' | 'relative';
    dateFormat: string;
  };
  filter: {
    defaultProtocols: Protocol[];
    rememberFilters: boolean;
  };
  export: {
    defaultFormat: 'json' | 'har' | 'pcap' | 'csv';
    defaultPath: string;
    includeRawData: boolean;
  };
  advanced: {
    logLevel: 'debug' | 'info' | 'warn' | 'error';
    enablePerformanceMonitor: boolean;
  };
}
```

#### 3.6.3 UI设计
- **布局**：侧边抽屉或模态对话框
- **分类**：
  - 抓包设置
  - 显示设置
  - 过滤设置
  - 导出设置
  - 高级设置
- **控件**：
  - 数字输入框
  - 开关
  - 下拉选择
  - 文件路径选择器
  - 颜色选择器（主题）

### 3.7 替换规则管理面板 (ReplaceRulesPanel)

#### 3.7.1 功能描述
管理所有数据包替换规则，支持创建、编辑、启用/禁用、删除规则。

#### 3.7.2 组件结构
```typescript
interface ReplaceRulesPanelProps {
  visible: boolean;
  onClose: () => void;
  rules: ReplaceRule[];
  onAddRule: (rule: ReplaceRule) => void;
  onUpdateRule: (rule: ReplaceRule) => void;
  onDeleteRule: (ruleId: string) => void;
  onToggleRule: (ruleId: string, enabled: boolean) => void;
}
```

#### 3.7.3 UI设计
- **布局**：侧边抽屉或模态对话框
- **规则列表**：
  - 规则名称
  - 匹配条件摘要
  - 替换内容摘要
  - 启用/禁用开关
  - 匹配次数统计
  - 操作按钮（编辑/删除）
- **规则编辑器**：
  - 匹配条件设置
  - 替换内容设置
  - 应用范围设置
  - 预览功能

#### 3.7.4 交互设计
- 打开方式：从工具栏或数据包列表打开
- 规则排序：按创建时间或匹配次数排序
- 批量操作：支持批量启用/禁用、删除
- 规则测试：支持测试规则匹配效果

### 3.8 状态栏 (StatusBar)

#### 3.8.1 功能描述
显示应用状态和统计信息。

#### 3.8.2 显示内容
- 已捕获数据包总数
- 过滤后数据包数量
- 内存使用情况
- 当前抓包状态
- 活跃替换规则数量
- 网络速度（可选）

#### 3.7.3 UI设计
- 水平分割显示各项信息
- 状态指示器：颜色编码
- 可点击查看详细信息

## 4. 交互流程设计

### 4.1 开始抓包流程

```
用户操作流程：
1. 使用外部工具注入DLL到目标进程
   ↓
2. 打开应用
   ↓
3. 点击"开始抓包"按钮
   ↓
4. 应用开始监听已注入DLL的进程
   ↓
5. 实时显示捕获的数据包列表
   ↓
6. 数据包自动更新（通过IPC事件）
```

### 4.2 查看数据包详情流程

```
用户操作流程：
1. 在数据包列表中点击数据包
   ↓
2. 详情面板展开/显示
   ↓
3. 切换到不同标签页查看
   ↓
4. 复制/导出需要的信息
   ↓
5. 关闭详情面板（可选）
```

### 4.3 过滤数据包流程

```
用户操作流程：
1. 在过滤器栏设置过滤条件
   ↓
2. 实时应用过滤（防抖）
   ↓
3. 数据包列表更新
   ↓
4. 状态栏显示过滤后数量
   ↓
5. 保存过滤器预设（可选）
```

### 4.4 测试发包流程

```
用户操作流程：
1. 打开测试发包对话框
   ↓
2. 选择协议类型（TCP/UDP）
   ↓
3. 填写数据包内容
   - 目标地址：IP:Port
   - 源地址：IP:Port（可选）
   - 数据内容：十六进制或文本
   ↓
4. 可选：选择目标进程
   ↓
5. 可选：设置延迟发送时间
   ↓
6. 点击"发送"按钮
   ↓
7. 显示发送结果（成功/失败）
   ↓
8. 发送的数据包会出现在列表中（如果正在抓包）
```

### 4.5 替换数据包流程

```
用户操作流程：
1. 在封包拦截面板中输入待过滤特征
   ↓
2. 输入替换为特征（可选）
   ↓
3. 选择封包函数类型（Send/Recv等）
   ↓
4. 选择操作选项（仅记录/过滤/替换）
   ↓
5. 点击"加入特征"按钮
   ↓
6. 规则添加到特征规则表格
   ↓
7. 后续匹配的数据包自动应用规则
   ↓
8. 拦截日志显示匹配和操作记录
```

### 4.6 编辑数据包流程

```
用户操作流程：
1. 在数据包列表中选择数据包
   ↓
2. 点击"编辑"按钮
   ↓
3. 打开编辑对话框
   ↓
4. 修改数据包内容
   - 请求/响应头
   - 请求/响应体
   ↓
5. 预览修改后的效果
   ↓
6. 保存修改
   ↓
7. 数据包列表更新
```

### 4.7 重放数据包流程

```
用户操作流程：
1. 在数据包列表中选择发送方向的数据包
   ↓
2. 点击"重放"按钮
   ↓
3. 确认重放对话框（显示数据包信息）
   ↓
4. 选择重放选项
   - 立即发送
   - 延迟发送
   - 修改后发送
   ↓
5. 执行重放
   ↓
6. 显示重放结果
   ↓
7. 重放的数据包出现在列表中
```

### 4.8 导出数据流程

```
用户操作流程：
1. 选择要导出的数据包（可选，默认全部）
   ↓
2. 点击"导出"按钮
   ↓
3. 选择导出格式（JSON/HAR/PCAP/CSV）
   ↓
4. 选择保存路径
   ↓
5. 显示导出进度
   ↓
6. 导出完成提示
```

## 5. 状态管理设计

### 5.1 全局状态结构

```typescript
interface AppState {
  // 抓包相关
  captureStatus: 'idle' | 'capturing' | 'stopping';
  
  // 数据包相关
  packets: Packet[];
  filteredPackets: Packet[];
  selectedPacketId?: number;
  packetsLoading: boolean;
  
  // 过滤器相关
  filters: FilterConditions;
  filterPresets: FilterPreset[];
  
  // 测试发包相关
  testSendPanelVisible: boolean;
  testTemplates: TestPacketTemplate[];
  
  // 替换规则相关
  replaceRules: ReplaceRule[];
  replacePanelVisible: boolean;
  editingReplaceRule?: ReplaceRule;
  
  // UI状态
  ui: {
    detailPanelVisible: boolean;
    detailPanelHeight: number;
    settingsVisible: boolean;
    theme: 'light' | 'dark';
    testSendPanelVisible: boolean;
    replacePanelVisible: boolean;
  };
  
  // 设置
  settings: AppSettings;
  
  // 统计信息
  statistics: {
    totalPackets: number;
    filteredPackets: number;
    memoryUsage: number;
    networkSpeed: number;
    activeReplaceRules: number;
  };
}
```

### 5.2 状态管理方案

#### 方案1：Context API + useReducer（推荐）
- 适合中小型应用
- 无需额外依赖
- 代码简洁

```typescript
// AppContext.tsx
const AppContext = createContext<{
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
}>();

// AppReducer.ts
function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'START_CAPTURE':
      return { ...state, captureStatus: 'capturing' };
    case 'STOP_CAPTURE':
      return { ...state, captureStatus: 'idle' };
    case 'ADD_PACKET':
      return { ...state, packets: [...state.packets, action.payload] };
    case 'SELECT_PACKET':
      return { ...state, selectedPacketId: action.payload };
    case 'SET_FILTERS':
      return { ...state, filters: action.payload };
    case 'SET_REPLACE_RULES':
      return { ...state, replaceRules: action.payload };
    case 'ADD_REPLACE_RULE':
      return { ...state, replaceRules: [...state.replaceRules, action.payload] };
    case 'UPDATE_REPLACE_RULE':
      return {
        ...state,
        replaceRules: state.replaceRules.map(rule =>
          rule.id === action.payload.id ? action.payload : rule
        ),
      };
    case 'TOGGLE_TEST_SEND_PANEL':
      return { ...state, ui: { ...state.ui, testSendPanelVisible: !state.ui.testSendPanelVisible } };
    case 'TOGGLE_REPLACE_PANEL':
      return { ...state, ui: { ...state.ui, replacePanelVisible: !state.ui.replacePanelVisible } };
    // ... 其他action
  }
}
```

#### 方案2：Zustand（备选）
- 如果状态管理变得复杂
- 更轻量级的替代方案
- 更好的TypeScript支持

### 5.3 数据流设计

```
Tauri后端 (Rust)
    ↓ IPC事件
React事件监听器
    ↓ dispatch action
状态管理器 (Reducer)
    ↓ 更新state
React组件
    ↓ 重新渲染
UI更新
```

## 6. 样式设计

### 6.1 设计系统

#### 6.1.1 颜色方案

**浅色主题**：
- 主色：`#1890ff`（蓝色）
- 成功：`#52c41a`（绿色）
- 警告：`#faad14`（橙色）
- 错误：`#f5222d`（红色）
- 背景：`#ffffff`
- 文本：`#000000d9`
- 边框：`#d9d9d9`

**深色主题**：
- 主色：`#177ddc`（蓝色）
- 成功：`#49aa19`（绿色）
- 警告：`#d89614`（橙色）
- 错误：`#dc4446`（红色）
- 背景：`#141414`
- 文本：`#ffffffd9`
- 边框：`#434343`

#### 6.1.2 字体规范
- 字体家族：`-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif`
- 字体大小：
  - 标题：16px / 18px / 20px
  - 正文：14px
  - 辅助文字：12px
  - 小号文字：10px

#### 6.1.3 间距规范
- 基础间距：4px
- 常用间距：8px, 12px, 16px, 24px, 32px

#### 6.1.4 圆角规范
- 小圆角：2px（按钮、输入框）
- 中圆角：4px（卡片）
- 大圆角：8px（面板）

### 6.2 组件样式

#### 6.2.1 按钮样式
- 主要按钮：主色背景，白色文字
- 次要按钮：白色背景，主色边框和文字
- 文本按钮：透明背景，主色文字
- 危险按钮：红色背景或红色文字

#### 6.2.2 输入框样式
- 标准输入框：1px边框，4px圆角
- 聚焦状态：主色边框，阴影效果
- 错误状态：红色边框

#### 6.2.3 表格样式
- 斑马纹：交替行背景色
- 悬停效果：行背景色变化
- 选中效果：主色边框或背景

### 6.3 动画效果

#### 6.3.1 过渡动画
- 淡入淡出：200ms ease
- 滑动：300ms ease-in-out
- 缩放：200ms ease

#### 6.3.2 加载动画
- 数据加载：旋转图标
- 进度条：线性动画
- 骨架屏：脉冲动画

## 7. 性能优化策略

### 7.1 渲染优化

#### 7.1.1 虚拟滚动
- 使用 `react-window` 实现数据包列表虚拟滚动
- 只渲染可见区域的数据包
- 预估高度，减少滚动抖动

#### 7.1.2 组件懒加载
- 详情面板按需加载
- 设置面板懒加载
- 大型组件使用 React.lazy

#### 7.1.3 记忆化
- 使用 `React.memo` 包装纯展示组件
- 使用 `useMemo` 缓存计算结果
- 使用 `useCallback` 缓存回调函数

### 7.2 数据优化

#### 7.2.1 数据分页
- 初始加载固定数量（如1000条）
- 滚动到底部时加载更多
- 使用虚拟滚动时预加载

#### 7.2.2 数据更新策略
- 批量更新：收集一段时间内的数据包，批量更新
- 防抖更新：使用防抖函数（500ms）减少更新频率
- 节流更新：限制更新频率，避免过度渲染

#### 7.2.3 数据缓存
- 使用 IndexedDB 缓存历史数据包
- 内存中只保留最近的数据包
- 按需加载历史数据

### 7.3 网络优化

#### 7.3.1 IPC通信优化
- 批量发送数据包，减少IPC调用次数
- 使用二进制格式传输大数据
- 压缩传输数据

#### 7.3.2 事件监听优化
- 使用事件委托减少监听器数量
- 及时清理不需要的事件监听器
- 使用防抖处理高频事件

## 8. 类型定义

### 8.1 核心类型

```typescript
// 协议类型
type Protocol = 'HTTP' | 'HTTPS' | 'TCP' | 'UDP' | 'OTHER';

// 数据包方向
type Direction = 'send' | 'receive';

// 抓包状态
type CaptureStatus = 'idle' | 'capturing' | 'stopping';

// 主题类型
type Theme = 'light' | 'dark' | 'auto';

// 时间戳显示模式
type TimestampMode = 'absolute' | 'relative';

// 导出格式
type ExportFormat = 'json' | 'har' | 'pcap' | 'csv';
```

### 8.2 数据包类型

```typescript
interface Packet {
  id: number;
  timestamp: number;
  processId: number;
  processName: string;
  protocol: Protocol;
  direction: Direction;
  srcAddr: string;
  dstAddr: string;
  size: number;
  rawData?: Uint8Array;
}

interface PacketDetail extends Packet {
  parsedData?: {
    tcp?: {
      seq: number;
      ack: number;
      flags: number;
    };
    udp?: {
      length: number;
      checksum: number;
    };
  };
}

// 测试包类型
interface TestPacket {
  id?: string;
  protocol: 'TCP' | 'UDP';
  dstAddr: string;  // 目标地址 IP:Port
  srcAddr?: string; // 源地址 IP:Port（可选）
  data: string | Uint8Array;  // 数据内容
  targetProcessId?: number;  // 可选：指定目标进程
  delay?: number;  // 延迟发送（毫秒）
  tcpFlags?: number;  // TCP标志位（仅TCP）
}

interface TestPacketTemplate {
  id: string;
  name: string;
  packet: TestPacket;
  createdAt: number;
  updatedAt: number;
}

// 数据包替换类型
interface PacketReplacement {
  // HTTP/HTTPS替换
  method?: string;
  url?: string;
  headers?: Record<string, string>;
  requestBody?: string | Uint8Array;
  responseBody?: string | Uint8Array;
  statusCode?: number;
  
  // TCP/UDP替换
  data?: Uint8Array;
  
  // 通用选项
  replaceDirection: 'send' | 'receive' | 'both';
}

interface ReplaceRule {
  id?: string;
  name: string;
  enabled: boolean;
  matchConditions: {
    protocol?: Protocol[];
    packetFunction?: PacketFunction[];
    direction?: 'send' | 'receive' | 'all';
    featurePattern?: string;  // 特征匹配模式（正则表达式）
  };
  replacement: PacketReplacement;
  applyTo: 'all' | 'first' | 'count';  // 应用范围
  applyCount?: number;  // 如果applyTo为count，指定数量
  createdAt?: number;
  updatedAt?: number;
}
```

### 8.3 进程类型

```typescript
interface Process {
  id: number;
  name: string;
  pid: number;
  architecture: 'x86' | 'x64';
  icon?: string;
  isCapturing?: boolean;
  packetCount?: number;
  memoryUsage?: number;
  cpuUsage?: number;
}
```

### 8.4 过滤器类型

```typescript
interface FilterConditions {
  protocols: Protocol[];
  direction?: 'send' | 'receive' | 'all';
  domain?: string;
  method?: string;
  statusCode?: number;
  minSize?: number;
  maxSize?: number;
  keyword?: string;
  useRegex?: boolean;
  searchIn: ('url' | 'header' | 'body')[];
  timeRange?: {
    start: number;
    end: number;
  };
}

interface FilterPreset {
  id: string;
  name: string;
  conditions: FilterConditions;
  createdAt: number;
}
```

### 8.5 设置类型

```typescript
interface AppSettings {
  capture: {
    maxPackets: number;
    autoCleanup: boolean;
    cleanupInterval: number;
    bufferSize: number;
    sampleRate?: number; // 采样率（0-1）
  };
  display: {
    theme: Theme;
    fontSize: number;
    showTimestamp: TimestampMode;
    dateFormat: string;
    timezone: string;
  };
  filter: {
    defaultProtocols: Protocol[];
    rememberFilters: boolean;
    autoApply: boolean;
  };
  export: {
    defaultFormat: ExportFormat;
    defaultPath: string;
    includeRawData: boolean;
    compress: boolean;
  };
  advanced: {
    logLevel: 'debug' | 'info' | 'warn' | 'error';
    enablePerformanceMonitor: boolean;
    enableDevTools: boolean;
  };
}
```

## 9. Tauri IPC通信设计

### 9.1 命令定义

```typescript
// 抓包相关命令（注意：DLL注入由外部工具完成）
interface CaptureCommands {
  startCapture(): Promise<void>;  // 开始监听已注入DLL的进程
  stopCapture(): Promise<void>;   // 停止监听
  getCaptureStatus(): Promise<CaptureStatus>;
}

// 数据包相关命令
interface PacketCommands {
  getPackets(filters?: FilterConditions): Promise<Packet[]>;
  getPacketDetail(id: number): Promise<PacketDetail>;
  clearPackets(): Promise<void>;
  getPacketCount(): Promise<number>;
  editPacket(id: number, changes: Partial<Packet>): Promise<Packet>;  // 编辑数据包
}

// 测试发包相关命令
interface TestSendCommands {
  sendTestPacket(packet: TestPacket): Promise<void>;
  getTestTemplates(): Promise<TestPacketTemplate[]>;
  saveTestTemplate(template: TestPacketTemplate): Promise<void>;
  deleteTestTemplate(id: string): Promise<void>;
}

// 数据包替换相关命令
interface PacketReplaceCommands {
  setReplaceRule(rule: ReplaceRule): Promise<string>;  // 返回规则ID
  removeReplaceRule(ruleId: string): Promise<void>;
  getReplaceRules(): Promise<ReplaceRule[]>;
  enableReplaceRule(ruleId: string, enabled: boolean): Promise<void>;
  replacePacket(packetId: number, replacement: PacketReplacement): Promise<void>;
}

// 导出相关命令
interface ExportCommands {
  exportPackets(
    format: ExportFormat,
    packetIds?: number[],
    path?: string
  ): Promise<string>;
  exportSelected(format: ExportFormat, path?: string): Promise<string>;
}

// 设置相关命令
interface SettingsCommands {
  getSettings(): Promise<AppSettings>;
  updateSettings(settings: Partial<AppSettings>): Promise<void>;
  resetSettings(): Promise<void>;
}
```

### 9.2 事件定义

```typescript
// 数据包捕获事件
interface PacketCapturedEvent {
  type: 'packet-captured';
  payload: Packet;
}

// 进程状态变化事件
interface ProcessStatusChangedEvent {
  type: 'process-status-changed';
  payload: {
    pid: number;
    status: 'started' | 'stopped' | 'crashed';
  };
}

// 抓包状态变化事件
interface CaptureStatusChangedEvent {
  type: 'capture-status-changed';
  payload: {
    pid: number;
    status: CaptureStatus;
  };
}

// 错误事件
interface ErrorEvent {
  type: 'error';
  payload: {
    code: string;
    message: string;
    details?: any;
  };
}

// 统计信息更新事件
interface StatisticsUpdatedEvent {
  type: 'statistics-updated';
  payload: {
    totalPackets: number;
    filteredPackets: number;
    memoryUsage: number;
    networkSpeed: number;
  };
}

// 数据包替换事件
interface PacketReplacedEvent {
  type: 'packet-replaced';
  payload: {
    originalPacketId: number;
    replacedPacketId: number;
    ruleId: string;
  };
}

// 测试包发送事件
interface TestPacketSentEvent {
  type: 'test-packet-sent';
  payload: {
    packetId: number;
    success: boolean;
    error?: string;
  };
}
```

### 9.3 IPC使用示例

```typescript
// 监听数据包事件
import { listen } from '@tauri-apps/api/event';

useEffect(() => {
  const unlisten = listen<Packet>('packet-captured', (event) => {
    dispatch({ type: 'ADD_PACKET', payload: event.payload });
  });

  return () => {
    unlisten.then(fn => fn());
  };
}, []);

// 调用命令
import { invoke } from '@tauri-apps/api/core';

const handleStartCapture = async () => {
  try {
    await invoke('start_capture');
    dispatch({ type: 'START_CAPTURE' });
  } catch (error) {
    message.error(`开始抓包失败: ${error}`);
  }
};

// 发送测试包
const handleSendTestPacket = async (packet: TestPacket) => {
  try {
    await invoke('send_test_packet', { packet });
    message.success('测试包发送成功');
  } catch (error) {
    message.error(`发送失败: ${error}`);
  }
};

// 设置替换规则
const handleSetReplaceRule = async (rule: ReplaceRule) => {
  try {
    const ruleId = await invoke('set_replace_rule', { rule });
    message.success('替换规则已设置');
    return ruleId;
  } catch (error) {
    message.error(`设置替换规则失败: ${error}`);
  }
};
```

## 10. 组件实现细节

### 10.1 虚拟滚动实现

```typescript
import { FixedSizeList as List } from 'react-window';

const PacketList: React.FC<PacketListProps> = ({ packets, ... }) => {
  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const packet = packets[index];
    return (
      <div style={style}>
        <PacketRow packet={packet} />
      </div>
    );
  };

  return (
    <List
      height={600}
      itemCount={packets.length}
      itemSize={50}
      width="100%"
    >
      {Row}
    </List>
  );
};
```

### 10.2 过滤器实现

```typescript
const usePacketFilter = (
  packets: Packet[],
  filters: FilterConditions
) => {
  return useMemo(() => {
    return packets.filter(packet => {
      // 协议过滤
      if (filters.protocols.length > 0 && !filters.protocols.includes(packet.protocol)) {
        return false;
      }

      // 方向过滤
      if (filters.direction && filters.direction !== 'all' && packet.direction !== filters.direction) {
        return false;
      }

      // 域名过滤
      if (filters.domain && !packet.domain?.includes(filters.domain)) {
        return false;
      }

      // 封包函数过滤
      if (
        filters.packetFunctions &&
        filters.packetFunctions.length > 0 &&
        packet.packetFunction &&
        !filters.packetFunctions.includes(packet.packetFunction)
      ) {
        return false;
      }

      // 大小过滤
      if (filters.minSize && packet.size < filters.minSize) {
        return false;
      }
      if (filters.maxSize && packet.size > filters.maxSize) {
        return false;
      }

      // 关键词搜索（在封包数据中搜索）
      if (filters.keyword) {
        const keyword = filters.useRegex
          ? filters.keyword
          : filters.keyword.toLowerCase();
        const searchIn = filters.searchIn || ['data'];
        
        let matched = false;
        if (searchIn.includes('data')) {
          const dataStr = packet.packetData || 
            (packet.rawData ? Array.from(packet.rawData).map(b => b.toString(16).padStart(2, '0')).join(' ') : '');
          if (filters.useRegex) {
            try {
              matched = new RegExp(keyword).test(dataStr);
            } catch {
              // 正则表达式无效，忽略
            }
          } else {
            matched = dataStr.toLowerCase().includes(keyword);
          }
        }

        if (!matched) {
          return false;
        }
      }

      return true;
    });
  }, [packets, filters]);
};
```

### 10.3 数据格式化工具

```typescript
// 格式化文件大小
export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
};

// 格式化时间戳
export const formatTimestamp = (
  timestamp: number,
  mode: TimestampMode = 'absolute'
): string => {
  if (mode === 'relative') {
    const now = Date.now();
    const diff = now - timestamp;
    if (diff < 1000) return '刚刚';
    if (diff < 60000) return `${Math.floor(diff / 1000)}秒前`;
    if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
    return `${Math.floor(diff / 3600000)}小时前`;
  }
  return new Date(timestamp).toLocaleString('zh-CN');
};

// 格式化协议名称
export const formatProtocol = (protocol: Protocol): string => {
  const map: Record<Protocol, string> = {
    HTTP: 'HTTP',
    HTTPS: 'HTTPS',
    TCP: 'TCP',
    UDP: 'UDP',
    OTHER: '其他',
  };
  return map[protocol] || protocol;
};

// 格式化HTTP状态码
export const formatStatusCode = (code?: number): string => {
  if (!code) return '-';
  const category = Math.floor(code / 100);
  const colors: Record<number, string> = {
    2: '#52c41a', // 成功 - 绿色
    3: '#1890ff', // 重定向 - 蓝色
    4: '#faad14', // 客户端错误 - 橙色
    5: '#f5222d', // 服务器错误 - 红色
  };
  return `<span style="color: ${colors[category] || '#000'}">${code}</span>`;
};
```

## 11. 错误处理

### 11.1 错误类型定义

```typescript
interface AppError {
  code: string;
  message: string;
  details?: any;
  timestamp: number;
}

// 错误代码
enum ErrorCode {
  PROCESS_NOT_FOUND = 'PROCESS_NOT_FOUND',
  INJECTION_FAILED = 'INJECTION_FAILED',
  PERMISSION_DENIED = 'PERMISSION_DENIED',
  NETWORK_ERROR = 'NETWORK_ERROR',
  EXPORT_FAILED = 'EXPORT_FAILED',
  UNKNOWN_ERROR = 'UNKNOWN_ERROR',
}
```

### 11.2 错误处理策略

- **用户友好提示**：将技术错误转换为用户可理解的提示
- **错误日志**：记录详细错误信息用于调试
- **错误恢复**：提供重试机制
- **错误边界**：使用 React Error Boundary 捕获组件错误

### 11.3 错误提示组件

```typescript
const ErrorMessage: React.FC<{ error: AppError }> = ({ error }) => {
  const getErrorMessage = (code: string): string => {
    const messages: Record<string, string> = {
      PROCESS_NOT_FOUND: '进程不存在或已退出',
      INJECTION_FAILED: 'DLL注入失败，请检查权限',
      PERMISSION_DENIED: '权限不足，请以管理员身份运行',
      NETWORK_ERROR: '网络错误，请检查网络连接',
      EXPORT_FAILED: '导出失败，请检查文件路径',
      UNKNOWN_ERROR: '未知错误，请查看日志',
    };
    return messages[code] || messages.UNKNOWN_ERROR;
  };

  return (
    <Alert
      message="错误"
      description={getErrorMessage(error.code)}
      type="error"
      showIcon
    />
  );
};
```

## 12. 测试策略

### 12.1 单元测试

- **组件测试**：使用 React Testing Library 测试组件渲染和交互
- **工具函数测试**：测试格式化、过滤等工具函数
- **Hook测试**：测试自定义Hook的逻辑

### 12.2 集成测试

- **流程测试**：测试完整的用户操作流程
- **IPC测试**：测试与Tauri后端的通信
- **性能测试**：测试大量数据包的处理性能

### 12.3 E2E测试

- **关键流程**：使用 Playwright 或 Cypress 测试关键用户流程
- **跨平台测试**：测试不同Windows版本的兼容性

## 13. 开发规范

### 13.1 代码规范

- **TypeScript严格模式**：启用所有严格检查
- **ESLint配置**：使用推荐的React和TypeScript规则
- **Prettier格式化**：统一代码格式
- **命名规范**：
  - 组件：PascalCase
  - 函数/变量：camelCase
  - 常量：UPPER_SNAKE_CASE
  - 类型/接口：PascalCase，以I开头或直接使用描述性名称

### 13.2 组件规范

- **单一职责**：每个组件只负责一个功能
- **Props类型**：明确定义所有Props类型
- **默认值**：为可选Props提供默认值
- **文档注释**：为复杂组件添加JSDoc注释

### 13.3 文件组织

```
src/
├── components/          # 组件目录
│   ├── MainPanel/
│   │   ├── index.tsx
│   │   ├── MainPanel.tsx
│   │   └── MainPanel.module.css
│   ├── PacketList/
│   └── ...
├── hooks/              # 自定义Hooks
│   ├── usePacketFilter.ts
│   ├── useCapture.ts
│   └── ...
├── utils/              # 工具函数
│   ├── formatters.ts
│   ├── validators.ts
│   └── ...
├── types/              # 类型定义
│   ├── packet.ts
│   ├── process.ts
│   └── ...
├── contexts/           # Context定义
│   ├── AppContext.tsx
│   └── ...
├── stores/             # 状态管理（如果使用Zustand）
├── styles/             # 全局样式
│   ├── variables.css
│   └── ...
└── App.tsx
```

## 14. 部署与构建

### 14.1 构建配置

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom'],
          'antd-vendor': ['antd'],
        },
      },
    },
  },
});
```

### 14.2 环境变量

```bash
# .env.development
VITE_API_URL=http://localhost:3000
VITE_LOG_LEVEL=debug

# .env.production
VITE_API_URL=
VITE_LOG_LEVEL=error
```

### 14.3 性能监控

- **性能指标**：监控FPS、内存使用、渲染时间
- **错误追踪**：集成错误追踪服务（如Sentry）
- **用户行为分析**：记录关键操作（可选）

## 15. 未来扩展

### 15.1 功能扩展

- **数据包重放**：支持重放HTTP请求
- **脚本支持**：支持自定义JavaScript脚本处理数据包
- **插件系统**：支持第三方插件扩展功能
- **协作功能**：支持多人协作分析（可选）

### 15.2 UI增强

- **数据可视化**：图表展示网络流量趋势
- **时间线视图**：可视化展示请求时间线
- **对比功能**：对比不同数据包
- **书签功能**：标记重要的数据包

### 15.3 性能优化

- **Web Workers**：使用Web Workers处理大量数据
- **IndexedDB优化**：优化数据存储和检索
- **增量更新**：只更新变化的数据包

---

**文档版本**：v1.0  
**最后更新**：2024年  
**维护者**：开发团队

## 附录：参考资源

### A.1 UI组件库
- [Ant Design 5.0 文档](https://ant.design/)
- [Ant Design Icons](https://ant.design/components/icon-cn/)

### A.2 React相关
- [React 19 文档](https://react.dev/)
- [React Hooks 文档](https://react.dev/reference/react)

### A.3 虚拟滚动
- [react-window](https://github.com/bvaughn/react-window)
- [react-virtualized](https://github.com/bvaughn/react-virtualized)

### A.4 状态管理
- [Zustand](https://github.com/pmndrs/zustand)
- [Context API](https://react.dev/reference/react/useContext)

### A.5 Tauri相关
- [Tauri 2.0 文档](https://v2.tauri.app/)
- [Tauri API 参考](https://v2.tauri.app/api/)

### A.6 工具库
- [dayjs](https://day.js.org/) - 日期处理
- [lodash](https://lodash.com/) - 工具函数库（按需引入）
- [clsx](https://github.com/lukeed/clsx) - className工具