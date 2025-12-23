import React from 'react';
import { Table } from 'antd';
import { useApp } from '../../contexts/AppContext';
import type { Packet } from '../../types';
import styles from './PacketList.module.css';

export const PacketList: React.FC = () => {
  const { state, dispatch } = useApp();
  const { filteredPackets, selectedPacketId, settings } = state;

  const handleSelectPacket = (packet: Packet) => {
    dispatch({
      type: 'SELECT_PACKET',
      payload: packet.id === selectedPacketId ? undefined : packet.id,
    });
  };

  const columns = [
    {
      title: '序号',
      dataIndex: 'id',
      key: 'id',
      width: 80,
      render: (_: any, __: Packet, index: number) => index + 1,
    },
    {
      title: 'IP地址',
      dataIndex: 'dstAddr',
      key: 'dstAddr',
      width: 180,
      render: (_: any, record: Packet) => record.dstAddr || '-',
    },
    {
      title: '封包函数',
      dataIndex: 'packetFunction',
      key: 'packetFunction',
      width: 100,
      render: (func?: string) => func || '-',
    },
    {
      title: '包长',
      dataIndex: 'size',
      key: 'size',
      width: 100,
      render: (size: number) => size,
    },
    {
      title: '套接字',
      dataIndex: 'socket',
      key: 'socket',
      width: 100,
      render: (socket?: number) => socket || '-',
    },
    {
      title: '封包数据',
      dataIndex: 'packetData',
      key: 'packetData',
      width: 300,
      ellipsis: true,
      render: (data?: string, record: Packet) => {
        if (data) return data;
        if (record.rawData) {
          return Array.from(record.rawData.slice(0, 20))
            .map((b) => b.toString(16).padStart(2, '0'))
            .join(' ') + (record.rawData.length > 20 ? '...' : '');
        }
        return '-';
      },
    },
  ];

  const rowClassName = (record: Packet) => {
    let className = '';
    if (record.id === selectedPacketId) {
      className += ` ${styles.selectedRow}`;
    }
    if (record.direction === 'send') {
      className += ` ${styles.sendRow}`;
    } else {
      className += ` ${styles.receiveRow}`;
    }
    return className.trim();
  };

  return (
    <div className={styles.packetList}>
      <Table
        columns={columns}
        dataSource={filteredPackets}
        rowKey="id"
        scroll={{ x: 1000, y: 'calc(100vh - 500px)' }}
        pagination={false}
        size="small"
        rowClassName={rowClassName}
        onRow={(record) => ({
          onClick: () => handleSelectPacket(record),
        })}
      />
    </div>
  );
};

