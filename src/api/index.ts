// API 抽象层 - 暂时不实现，只定义接口

import { invoke } from '@tauri-apps/api/core';
import type {
  Packet,
  PacketDetail,
  TestPacket,
  TestPacketTemplate,
  ReplaceRule,
  FilterConditions,
  ExportFormat,
  AppSettings,
  CaptureStatus,
} from '../types';

// 抓包相关命令
export const captureApi = {
  startCapture: async (): Promise<void> => {
    await invoke('start_capture');
  },
  stopCapture: async (): Promise<void> => {
    await invoke('stop_capture');
  },
  getCaptureStatus: async (): Promise<CaptureStatus> => {
    const status = await invoke<string>('get_capture_status');
    return status as CaptureStatus;
  },
};

// 数据包相关命令
export const packetApi = {
  getPackets: async (_filters?: FilterConditions): Promise<Packet[]> => {
    // TODO: 实现获取数据包列表
    return [];
  },
  getPacketDetail: async (_id: number): Promise<PacketDetail> => {
    // TODO: 实现获取数据包详情
    throw new Error('Not implemented');
  },
  clearPackets: async (): Promise<void> => {
    // TODO: 实现清空数据包
    console.log('clearPackets');
  },
  getPacketCount: async (): Promise<number> => {
    // TODO: 实现获取数据包数量
    return 0;
  },
  editPacket: async (_id: number, _changes: Partial<Packet>): Promise<Packet> => {
    // TODO: 实现编辑数据包
    throw new Error('Not implemented');
  },
};

// 测试发包相关命令
export const testSendApi = {
  sendTestPacket: async (packet: TestPacket): Promise<void> => {
    // TODO: 实现发送测试包
    console.log('sendTestPacket', packet);
  },
  getTestTemplates: async (): Promise<TestPacketTemplate[]> => {
    // TODO: 实现获取测试模板
    return [];
  },
  saveTestTemplate: async (template: TestPacketTemplate): Promise<void> => {
    // TODO: 实现保存测试模板
    console.log('saveTestTemplate', template);
  },
  deleteTestTemplate: async (id: string): Promise<void> => {
    // TODO: 实现删除测试模板
    console.log('deleteTestTemplate', id);
  },
};

// 数据包替换相关命令
export const packetReplaceApi = {
  setReplaceRule: async (rule: ReplaceRule): Promise<string> => {
    // TODO: 实现设置替换规则
    console.log('setReplaceRule', rule);
    return 'rule-id';
  },
  removeReplaceRule: async (ruleId: string): Promise<void> => {
    // TODO: 实现删除替换规则
    console.log('removeReplaceRule', ruleId);
  },
  getReplaceRules: async (): Promise<ReplaceRule[]> => {
    // TODO: 实现获取替换规则列表
    return [];
  },
  enableReplaceRule: async (ruleId: string, enabled: boolean): Promise<void> => {
    // TODO: 实现启用/禁用替换规则
    console.log('enableReplaceRule', ruleId, enabled);
  },
  replacePacket: async (packetId: number, replacement: any): Promise<void> => {
    // TODO: 实现替换数据包
    console.log('replacePacket', packetId, replacement);
  },
};

// 导出相关命令
export const exportApi = {
  exportPackets: async (
    format: ExportFormat,
    packetIds?: number[],
    path?: string
  ): Promise<string> => {
    // TODO: 实现导出数据包
    console.log('exportPackets', format, packetIds, path);
    return '/path/to/exported/file';
  },
  exportSelected: async (format: ExportFormat, path?: string): Promise<string> => {
    // TODO: 实现导出选中的数据包
    console.log('exportSelected', format, path);
    return '/path/to/exported/file';
  },
};

// 设置相关命令
export const settingsApi = {
  getSettings: async (): Promise<AppSettings> => {
    // TODO: 实现获取设置
    return getDefaultSettings();
  },
  updateSettings: async (settings: Partial<AppSettings>): Promise<void> => {
    // TODO: 实现更新设置
    console.log('updateSettings', settings);
  },
  resetSettings: async (): Promise<void> => {
    // TODO: 实现重置设置
    console.log('resetSettings');
  },
};

// 默认设置
function getDefaultSettings(): AppSettings {
  return {
    capture: {
      maxPackets: 10000,
      autoCleanup: true,
      cleanupInterval: 60,
      bufferSize: 1024 * 1024,
      sampleRate: 1.0,
    },
    display: {
      theme: 'light',
      fontSize: 14,
      showTimestamp: 'absolute',
      dateFormat: 'YYYY-MM-DD HH:mm:ss',
      timezone: 'Asia/Shanghai',
    },
    filter: {
      defaultProtocols: [],
      rememberFilters: false,
      autoApply: true,
    },
    export: {
      defaultFormat: 'json',
      defaultPath: '',
      includeRawData: false,
      compress: false,
    },
    advanced: {
      logLevel: 'info',
      enablePerformanceMonitor: false,
      enableDevTools: false,
    },
  };
}

