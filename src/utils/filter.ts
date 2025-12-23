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

      let matched = false;

      // 搜索封包数据
      if (packet.packetData) {
        if (filters.useRegex) {
          try {
            matched = new RegExp(keyword).test(packet.packetData);
          } catch {
            // 正则表达式无效，忽略
          }
        } else {
          matched = packet.packetData.toLowerCase().includes(keyword);
        }
      }

      // 搜索原始数据
      if (!matched && packet.rawData) {
        const rawStr = Array.from(packet.rawData)
          .map((b) => b.toString(16).padStart(2, '0'))
          .join(' ');
        if (filters.useRegex) {
          try {
            matched = new RegExp(keyword).test(rawStr);
          } catch {
            // 正则表达式无效，忽略
          }
        } else {
          matched = rawStr.toLowerCase().includes(keyword);
        }
      }

      if (!matched) {
        return false;
      }
    }

    return true;
  });
};

