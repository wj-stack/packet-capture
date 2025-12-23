import React, { useMemo } from 'react';
import { Drawer, Tabs, Descriptions, Button, Space, Collapse } from 'antd';
import { EditOutlined, SendOutlined, SwapOutlined, CloseOutlined } from '@ant-design/icons';
import { useApp } from '../../contexts/AppContext';
import { packetApi } from '../../api';
import type { PacketDetail } from '../../types';
import styles from './PacketDetailPanel.module.css';

const { Panel } = Collapse;

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
      key: 'request',
      label: '请求',
      children: selectedPacket.protocol === 'HTTP' || selectedPacket.protocol === 'HTTPS' ? (
        <div>
          <div className={styles.section}>
            <h4>请求行</h4>
            <pre className={styles.code}>
              {selectedPacket.method} {selectedPacket.url} HTTP/1.1
            </pre>
          </div>
          {selectedPacket.headers && (
            <div className={styles.section}>
              <h4>请求头</h4>
              <Collapse>
                <Panel header="查看请求头" key="1">
                  <pre className={styles.code}>
                    {JSON.stringify(selectedPacket.headers, null, 2)}
                  </pre>
                </Panel>
              </Collapse>
            </div>
          )}
          {selectedPacket.requestBody && (
            <div className={styles.section}>
              <h4>请求体</h4>
              <pre className={styles.code}>{selectedPacket.requestBody}</pre>
            </div>
          )}
        </div>
      ) : (
        <div>非HTTP协议，无请求信息</div>
      ),
    },
    {
      key: 'response',
      label: '响应',
      children: selectedPacket.protocol === 'HTTP' || selectedPacket.protocol === 'HTTPS' ? (
        <div>
          {selectedPacket.statusCode && (
            <div className={styles.section}>
              <h4>状态行</h4>
              <pre className={styles.code}>HTTP/1.1 {selectedPacket.statusCode}</pre>
            </div>
          )}
          {selectedPacket.headers && (
            <div className={styles.section}>
              <h4>响应头</h4>
              <Collapse>
                <Panel header="查看响应头" key="1">
                  <pre className={styles.code}>
                    {JSON.stringify(selectedPacket.headers, null, 2)}
                  </pre>
                </Panel>
              </Collapse>
            </div>
          )}
          {selectedPacket.responseBody && (
            <div className={styles.section}>
              <h4>响应体</h4>
              <pre className={styles.code}>{selectedPacket.responseBody}</pre>
            </div>
          )}
        </div>
      ) : (
        <div>非HTTP协议，无响应信息</div>
      ),
    },
    {
      key: 'raw',
      label: '原始数据',
      children: (
        <div>
          <h4>十六进制视图</h4>
          <pre className={styles.code}>
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
        <div className={styles.header}>
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

