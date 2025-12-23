import type { Packet, FilterConditions } from '../types';

// 数据包过滤 Hook
export const usePacketFilter = (
  packets: Packet[],
  filters: FilterConditions
): Packet[] => {
  return packets.filter((packet) => {
    // 协议过滤
    if (filters.protocols.length > 0 && !filters.protocols.includes(packet.protocol)) {
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

    // 方向过滤
    if (filters.direction && filters.direction !== 'all' && packet.direction !== filters.direction) {
      return false;
    }

    // 域名过滤
    if (filters.domain && !packet.domain?.includes(filters.domain)) {
      return false;
    }

    // HTTP方法过滤
    if (filters.method && packet.method !== filters.method) {
      return false;
    }

    // 状态码过滤
    if (filters.statusCode && packet.statusCode !== filters.statusCode) {
      return false;
    }

    // 大小过滤
    if (filters.minSize && packet.size < filters.minSize) {
      return false;
    }
    if (filters.maxSize && packet.size > filters.maxSize) {
      return false;
    }

    // 时间范围过滤
    if (filters.timeRange) {
      if (packet.timestamp < filters.timeRange.start || packet.timestamp > filters.timeRange.end) {
        return false;
      }
    }

    // 关键词搜索
    if (filters.keyword) {
      const keyword = filters.useRegex
        ? filters.keyword
        : filters.keyword.toLowerCase();
      const searchIn = filters.searchIn || ['url', 'header', 'body'];

      let matched = false;

      if (searchIn.includes('url') && packet.url) {
        if (filters.useRegex) {
          try {
            matched = new RegExp(keyword).test(packet.url);
          } catch {
            // 正则表达式无效，忽略
          }
        } else {
          matched = packet.url.toLowerCase().includes(keyword);
        }
      }

      if (!matched && searchIn.includes('header') && packet.headers) {
        const headerStr = JSON.stringify(packet.headers);
        if (filters.useRegex) {
          try {
            matched = new RegExp(keyword).test(headerStr);
          } catch {
            // 正则表达式无效，忽略
          }
        } else {
          matched = headerStr.toLowerCase().includes(keyword);
        }
      }

      if (!matched && searchIn.includes('body')) {
        const bodyStr = (packet.requestBody || packet.responseBody || '').toString();
        if (filters.useRegex) {
          try {
            matched = new RegExp(keyword).test(bodyStr);
          } catch {
            // 正则表达式无效，忽略
          }
        } else {
          matched = bodyStr.toLowerCase().includes(keyword);
        }
      }

      if (!matched) {
        return false;
      }
    }

    return true;
  });
};

