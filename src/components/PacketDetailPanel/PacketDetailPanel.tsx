import React, { useMemo } from 'react';
import { Drawer, Tabs, Descriptions, Button, Space } from 'antd';
import { EditOutlined, SendOutlined, SwapOutlined, CloseOutlined } from '@ant-design/icons';
import { useApp } from '../../contexts/AppContext';
import type { PacketDetail } from '../../types';

export const PacketDetailPanel: React.FC = () => {
  const { state, dispatch } = useApp();
  const { selectedPacketId, packets, ui } = state;

  const selectedPacket = useMemo(() => {
    if (!selectedPacketId) return null;
    return packets.find((p) => p.id === selectedPacketId) as PacketDetail | null;
  }, [selectedPacketId, packets]);

  const handleClose = () => {
    dispatch({ type: 'SELECT_PACKET', payload: undefined });
  };

  const handleEdit = () => {
    if (!selectedPacket) return;
    // TODO: 实现编辑功能
    console.log('编辑数据包', selectedPacket);
  };

  const handleReplay = () => {
    if (!selectedPacket) return;
    // TODO: 实现重放功能
    console.log('重放数据包', selectedPacket);
  };

  const handleReplace = () => {
    if (!selectedPacket) return;
    // TODO: 实现替换功能
    console.log('替换数据包', selectedPacket);
  };

  if (!selectedPacket) return null;

  const tabItems = [
    {
      key: 'overview',
      label: '概览',
      children: (
        <Descriptions column={2} bordered>
          <Descriptions.Item label="ID">{selectedPacket.id}</Descriptions.Item>
          <Descriptions.Item label="时间戳">
            {new Date(selectedPacket.timestamp).toLocaleString('zh-CN')}
          </Descriptions.Item>
          <Descriptions.Item label="进程名">{selectedPacket.processName}</Descriptions.Item>
          <Descriptions.Item label="进程ID">{selectedPacket.processId}</Descriptions.Item>
          <Descriptions.Item label="协议">{selectedPacket.protocol}</Descriptions.Item>
          <Descriptions.Item label="方向">{selectedPacket.direction}</Descriptions.Item>
          <Descriptions.Item label="源地址">{selectedPacket.srcAddr}</Descriptions.Item>
          <Descriptions.Item label="目标地址">{selectedPacket.dstAddr}</Descriptions.Item>
          <Descriptions.Item label="大小">{selectedPacket.size} bytes</Descriptions.Item>
          {selectedPacket.duration && (
            <Descriptions.Item label="耗时">{selectedPacket.duration}ms</Descriptions.Item>
          )}
        </Descriptions>
      ),
    },
    {
      key: 'data',
      label: '封包数据',
      children: (
        <div>
          {selectedPacket.packetData && (
            <div className="mb-4">
              <h4 className="mb-2 font-semibold">封包数据（十六进制）</h4>
              <pre className="bg-[#f5f5f5] p-3 rounded overflow-x-auto font-mono text-xs leading-relaxed">{selectedPacket.packetData}</pre>
            </div>
          )}
          {selectedPacket.packetFunction && (
            <div className="mb-4">
              <h4 className="mb-2 font-semibold">封包函数</h4>
              <pre className="bg-[#f5f5f5] p-3 rounded overflow-x-auto font-mono text-xs leading-relaxed">{selectedPacket.packetFunction}</pre>
            </div>
          )}
        </div>
      ),
    },
    {
      key: 'raw',
      label: '原始数据',
      children: (
        <div>
          <h4 className="mb-2 font-semibold">十六进制视图</h4>
          <pre className="bg-[#f5f5f5] p-3 rounded overflow-x-auto font-mono text-xs leading-relaxed">
            {selectedPacket.rawData
              ? Array.from(selectedPacket.rawData)
                  .map((b) => b.toString(16).padStart(2, '0'))
                  .join(' ')
              : '无原始数据'}
          </pre>
        </div>
      ),
    },
  ];

  return (
    <Drawer
      title={
        <div className="flex justify-between items-center w-full">
          <span>数据包详情</span>
          <Space>
            <Button size="small" icon={<EditOutlined />} onClick={handleEdit}>
              编辑
            </Button>
            <Button
              size="small"
              icon={<SendOutlined />}
              onClick={handleReplay}
              disabled={selectedPacket.direction === 'receive'}
            >
              重放
            </Button>
            <Button size="small" icon={<SwapOutlined />} onClick={handleReplace}>
              替换
            </Button>
            <Button size="small" icon={<CloseOutlined />} onClick={handleClose}>
              关闭
            </Button>
          </Space>
        </div>
      }
      open={ui.detailPanelVisible}
      onClose={handleClose}
      placement="bottom"
      height={ui.detailPanelHeight}
      mask={false}
      style={{ position: 'absolute' }}
    >
      <Tabs items={tabItems} />
    </Drawer>
  );
};

