import type { Protocol, TimestampMode } from '../types';

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

// 获取协议颜色
export const getProtocolColor = (protocol: Protocol): string => {
  const colors: Record<Protocol, string> = {
    HTTP: '#1890ff',
    HTTPS: '#52c41a',
    TCP: '#faad14',
    UDP: '#722ed1',
    OTHER: '#8c8c8c',
  };
  return colors[protocol] || '#8c8c8c';
};

// 格式化HTTP状态码
export const formatStatusCode = (code?: number): string => {
  if (!code) return '-';
  return code.toString();
};

// 获取状态码颜色
export const getStatusCodeColor = (code?: number): string => {
  if (!code) return '#8c8c8c';
  const category = Math.floor(code / 100);
  const colors: Record<number, string> = {
    2: '#52c41a', // 成功 - 绿色
    3: '#1890ff', // 重定向 - 蓝色
    4: '#faad14', // 客户端错误 - 橙色
    5: '#f5222d', // 服务器错误 - 红色
  };
  return colors[category] || '#8c8c8c';
};

// 格式化方向
export const formatDirection = (direction: 'send' | 'receive'): string => {
  return direction === 'send' ? '↑ 发送' : '↓ 接收';
};

// 格式化耗时
export const formatDuration = (duration?: number): string => {
  if (!duration) return '-';
  if (duration < 1000) return `${duration}ms`;
  return `${(duration / 1000).toFixed(2)}s`;
};

