// 协议类型
export type Protocol = 'HTTP' | 'HTTPS' | 'TCP' | 'UDP' | 'OTHER';

// 数据包方向
export type Direction = 'send' | 'receive';

// 封包函数类型
export type PacketFunction =
  | 'Send'
  | 'Recv'
  | 'SendTo'
  | 'RecvFrom'
  | 'WSASend'
  | 'WSARecv'
  | 'WSASendTo'
  | 'WSARecvFrom';

// 抓包状态
export type CaptureStatus = 'idle' | 'capturing' | 'stopping';

// 主题类型
export type Theme = 'light' | 'dark' | 'auto';

// 时间戳显示模式
export type TimestampMode = 'absolute' | 'relative';

// 导出格式
export type ExportFormat = 'json' | 'har' | 'pcap' | 'csv';

// 数据包接口
export interface Packet {
  id: number;
  timestamp: number;
  processId: number;
  processName: string;
  protocol: Protocol;
  direction: Direction;
  srcAddr: string;
  dstAddr: string;
  size: number;
  socket?: number; // 套接字
  packetFunction?: PacketFunction; // 封包函数
  packetData?: string; // 封包数据（十六进制字符串）
  duration?: number;
  rawData?: Uint8Array;
}

// 数据包详情
export interface PacketDetail extends Packet {
  parsedData?: {
    http?: {
      version: string;
      headers: Record<string, string>;
      body?: string;
    };
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
export interface TestPacket {
  id?: string;
  protocol: Protocol;
  method?: string;
  url: string;
  headers?: Record<string, string>;
  body?: string | Uint8Array;
  targetProcessId?: number;
  delay?: number;
  srcAddr?: string;
  dstAddr?: string;
  tcpFlags?: number;
}

// 测试包模板
export interface TestPacketTemplate {
  id: string;
  name: string;
  packet: TestPacket;
  createdAt: number;
  updatedAt: number;
}

// 数据包替换类型
export interface PacketReplacement {
  method?: string;
  url?: string;
  headers?: Record<string, string>;
  requestBody?: string | Uint8Array;
  responseBody?: string | Uint8Array;
  statusCode?: number;
  data?: Uint8Array;
  replaceDirection: 'send' | 'receive' | 'both';
}

// 替换规则
export interface ReplaceRule {
  id?: string;
  name: string;
  enabled: boolean;
  matchConditions: {
    protocol?: Protocol[];
    direction?: 'send' | 'receive' | 'all';
    urlPattern?: string;
    domain?: string;
    method?: string;
    statusCode?: number;
    headerMatch?: Record<string, string>;
  };
  replacement: PacketReplacement;
  applyTo: 'all' | 'first' | 'count';
  applyCount?: number;
  createdAt?: number;
  updatedAt?: number;
}

// 过滤器条件
export interface FilterConditions {
  protocols: Protocol[];
  direction?: 'send' | 'receive' | 'all';
  packetFunctions?: PacketFunction[]; // 封包函数过滤
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

// 封包特征规则
export interface PacketFeatureRule {
  id: string;
  type: PacketFunction;
  feature: string; // 待过滤特征
  replacement?: string; // 替换为特征
  onlyRecord: boolean; // 仅记录次数
  filter: boolean; // 过滤
  replace: boolean; // 替换
  matchCount?: number; // 匹配次数
}

// 过滤器预设
export interface FilterPreset {
  id: string;
  name: string;
  conditions: FilterConditions;
  createdAt: number;
}

// 应用设置
export interface AppSettings {
  capture: {
    maxPackets: number;
    autoCleanup: boolean;
    cleanupInterval: number;
    bufferSize: number;
    sampleRate?: number;
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

// 应用状态
export interface AppState {
  captureStatus: CaptureStatus;
  packets: Packet[];
  filteredPackets: Packet[];
  selectedPacketId?: number;
  packetsLoading: boolean;
  filters: FilterConditions;
  filterPresets: FilterPreset[];
  testSendPanelVisible: boolean;
  testTemplates: TestPacketTemplate[];
  replaceRules: ReplaceRule[];
  replacePanelVisible: boolean;
  editingReplaceRule?: ReplaceRule;
  ui: {
    detailPanelVisible: boolean;
    detailPanelHeight: number;
    settingsVisible: boolean;
    theme: 'light' | 'dark';
    testSendPanelVisible: boolean;
    replacePanelVisible: boolean;
  };
  settings: AppSettings;
  statistics: {
    totalPackets: number;
    filteredPackets: number;
    memoryUsage: number;
    networkSpeed: number;
    activeReplaceRules: number;
  };
}

// Action 类型
export type AppAction =
  | { type: 'START_CAPTURE' }
  | { type: 'STOP_CAPTURE' }
  | { type: 'SET_CAPTURE_STATUS'; payload: CaptureStatus }
  | { type: 'ADD_PACKET'; payload: Packet }
  | { type: 'ADD_PACKETS'; payload: Packet[] }
  | { type: 'CLEAR_PACKETS' }
  | { type: 'SELECT_PACKET'; payload: number | undefined }
  | { type: 'SET_FILTERS'; payload: FilterConditions }
  | { type: 'SET_FILTERED_PACKETS'; payload: Packet[] }
  | { type: 'SET_REPLACE_RULES'; payload: ReplaceRule[] }
  | { type: 'ADD_REPLACE_RULE'; payload: ReplaceRule }
  | { type: 'UPDATE_REPLACE_RULE'; payload: ReplaceRule }
  | { type: 'DELETE_REPLACE_RULE'; payload: string }
  | { type: 'TOGGLE_TEST_SEND_PANEL' }
  | { type: 'TOGGLE_REPLACE_PANEL' }
  | { type: 'TOGGLE_SETTINGS' }
  | { type: 'SET_DETAIL_PANEL_VISIBLE'; payload: boolean }
  | { type: 'SET_DETAIL_PANEL_HEIGHT'; payload: number }
  | { type: 'UPDATE_STATISTICS'; payload: Partial<AppState['statistics']> }
  | { type: 'UPDATE_SETTINGS'; payload: Partial<AppSettings> };

