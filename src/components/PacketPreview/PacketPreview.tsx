import React, { useMemo, useEffect } from 'react';
import { Select, Checkbox, Input } from 'antd';
import { useApp } from '../../contexts/AppContext';

const { Option } = Select;
const { TextArea } = Input;

const DECODE_OPTIONS = ['UTF-8', 'GBK', 'ASCII', 'Hex'];

export const PacketPreview: React.FC = () => {
  const { state } = useApp();
  const { selectedPacketId, packets } = state;
  const [decodeFormat, setDecodeFormat] = React.useState('UTF-8');
  const [realtimePreview, setRealtimePreview] = React.useState(false);
  const [rawData, setRawData] = React.useState('');

  const selectedPacket = useMemo(() => {
    if (!selectedPacketId) return null;
    return packets.find((p) => p.id === selectedPacketId);
  }, [selectedPacketId, packets]);

  // 更新原始数据
  useEffect(() => {
    if (!selectedPacket) {
      setRawData('');
      return;
    }

    const data = selectedPacket.packetData || selectedPacket.rawData;
    if (!data) {
      setRawData('');
      return;
    }

    if (typeof data === 'string') {
      setRawData(data);
    } else {
      // 将 Uint8Array 转换为十六进制字符串
      setRawData(
        Array.from(data)
          .map((b) => b.toString(16).padStart(2, '0'))
          .join(' ')
      );
    }
  }, [selectedPacket]);

  // 计算解密后的内容
  const decodedContent = useMemo(() => {
    if (!rawData) return '';

    if (decodeFormat === 'Hex') {
      return rawData;
    }

    try {
      // 将十六进制字符串转换为字节数组
      const bytes = rawData
        .split(' ')
        .filter((s) => s.trim())
        .map((s) => parseInt(s.trim(), 16))
        .filter((n) => !isNaN(n));

      if (bytes.length === 0) return '';

      const buffer = new Uint8Array(bytes);
      return decodeBuffer(buffer, decodeFormat);
    } catch {
      return '';
    }
  }, [rawData, decodeFormat]);

  // 实时预览：当原始数据变化时自动更新解密内容
  useEffect(() => {
    if (realtimePreview && selectedPacket) {
      // 实时预览模式下，数据会自动更新
    }
  }, [realtimePreview, selectedPacket]);

  const decodeBuffer = (buffer: Uint8Array, format: string): string => {
    try {
      if (format === 'UTF-8') {
        return new TextDecoder('utf-8').decode(buffer);
      } else if (format === 'GBK') {
        // GBK 解码需要特殊处理，这里简化处理
        try {
          return new TextDecoder('gbk').decode(buffer);
        } catch {
          // 如果GBK解码失败，尝试UTF-8
          return new TextDecoder('utf-8').decode(buffer);
        }
      } else if (format === 'ASCII') {
        return Array.from(buffer)
          .map((b) => (b >= 32 && b <= 126 ? String.fromCharCode(b) : '.'))
          .join('');
      }
      return buffer.toString();
    } catch {
      return Array.from(buffer)
        .map((b) => b.toString(16).padStart(2, '0'))
        .join(' ');
    }
  };

  return (
    <div className="px-2 sm:px-3 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded flex-1 flex flex-col min-h-0 shadow-sm">
      <div className="font-semibold mb-1.5 text-xs text-center px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-gray-900 dark:text-gray-100">封包预览</div>
      <div className="flex-1 flex flex-col md:flex-row gap-2 min-h-0 items-stretch">
        <TextArea
          value={rawData}
          onChange={(e) => setRawData(e.target.value)}
          className="flex-1 font-mono text-xs resize-none h-full bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 border-gray-200 dark:border-gray-700 min-h-[100px] md:min-h-0"
          placeholder="原始数据（十六进制）"
          readOnly={!!selectedPacket}
        />
        <div className="flex flex-row md:flex-col items-center justify-center gap-2 px-2 min-w-full md:min-w-[100px] md:max-w-[100px]">
          <div className="text-xs text-gray-600 dark:text-gray-400 whitespace-nowrap">解密为=&gt;</div>
          <Select
            value={decodeFormat}
            onChange={setDecodeFormat}
            className="w-full md:w-full"
            size="small"
          >
            {DECODE_OPTIONS.map((opt) => (
              <Option key={opt} value={opt}>
                {opt}
              </Option>
            ))}
          </Select>
          <Checkbox
            checked={realtimePreview}
            onChange={(e) => setRealtimePreview(e.target.checked)}
            className="whitespace-nowrap text-xs"
          >
            实时预览
          </Checkbox>
        </div>
        <TextArea
          value={decodedContent}
          readOnly
          className="flex-1 font-mono text-xs resize-none h-full bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 border-gray-200 dark:border-gray-700 min-h-[100px] md:min-h-0"
          placeholder="解密后的数据"
        />
      </div>
    </div>
  );
};

