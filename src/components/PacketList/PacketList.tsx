import React, { useRef, useEffect, useState } from 'react';
import { Table, Tag, Tooltip } from 'antd';
import { 
  SendOutlined, 
  DownloadOutlined, 
  ClockCircleOutlined,
  DatabaseOutlined 
} from '@ant-design/icons';
import { useApp } from '../../contexts/AppContext';
import type { Packet } from '../../types';

export const PacketList: React.FC = () => {
  const { state, dispatch } = useApp();
  const { filteredPackets, selectedPacketId } = state;
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollY, setScrollY] = useState<number | undefined>(550);

  const handleSelectPacket = (packet: Packet) => {
    const newSelectedId = packet.id === selectedPacketId ? undefined : packet.id;
    dispatch({
      type: 'SELECT_PACKET',
      payload: newSelectedId,
    });
  };

  // 格式化时间戳
  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp);
    const hours = date.getHours().toString().padStart(2, '0');
    const minutes = date.getMinutes().toString().padStart(2, '0');
    const seconds = date.getSeconds().toString().padStart(2, '0');
    const milliseconds = date.getMilliseconds().toString().padStart(3, '0');
    return `${hours}:${minutes}:${seconds}.${milliseconds}`;
  };

  // 格式化大小
  const formatSize = (size: number) => {
    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
    return `${(size / (1024 * 1024)).toFixed(2)} MB`;
  };

  // 获取协议颜色
  const getProtocolColor = (protocol: string) => {
    switch (protocol) {
      case 'TCP':
        return 'blue';
      case 'UDP':
        return 'green';
      case 'HTTP':
        return 'orange';
      case 'HTTPS':
        return 'purple';
      default:
        return 'default';
    }
  };

  const columns = [
    {
      title: '#',
      dataIndex: 'id',
      key: 'id',
      width: 60,
      fixed: 'left' as const,
      render: (_: any, __: Packet, index: number) => (
        <span className="font-semibold text-[12px] text-gray-500 dark:text-gray-400">{index + 1}</span>
      ),
    },
    {
      title: '时间',
      dataIndex: 'timestamp',
      key: 'timestamp',
      width: 120,
      fixed: 'left' as const,
      render: (timestamp: number) => (
        <Tooltip title={new Date(timestamp).toLocaleString('zh-CN')}>
          <span className="flex items-center gap-1 font-mono text-[12px] text-gray-600 dark:text-gray-400">
            <ClockCircleOutlined className="text-[12px] text-gray-500 dark:text-gray-500" />
            {formatTimestamp(timestamp)}
          </span>
        </Tooltip>
      ),
    },
    {
      title: '方向',
      dataIndex: 'direction',
      key: 'direction',
      width: 80,
      render: (direction: string) => (
        <Tag
          icon={direction === 'send' ? <SendOutlined /> : <DownloadOutlined />}
          color={direction === 'send' ? 'blue' : 'green'}
          className="m-0 font-medium inline-flex items-center gap-1"
        >
          {direction === 'send' ? '发送' : '接收'}
        </Tag>
      ),
    },
    {
      title: '协议',
      dataIndex: 'protocol',
      key: 'protocol',
      width: 80,
      render: (protocol: string) => (
        <Tag color={getProtocolColor(protocol)} className="m-0 font-medium font-mono">
          {protocol}
        </Tag>
      ),
    },
    {
      title: '进程',
      dataIndex: 'processName',
      key: 'processName',
      width: 120,
      ellipsis: true,
      render: (name: string, record: Packet) => (
        <Tooltip title={`PID: ${record.processId}`}>
          <span className="font-medium text-gray-900 dark:text-gray-100 text-[12px]">{name}</span>
        </Tooltip>
      ),
    },
    {
      title: '源地址',
      dataIndex: 'srcAddr',
      key: 'srcAddr',
      width: 160,
      ellipsis: true,
      render: (addr: string) => (
        <span className="font-mono text-[12px] text-gray-600 dark:text-gray-400">{addr || '-'}</span>
      ),
    },
    {
      title: '目标地址',
      dataIndex: 'dstAddr',
      key: 'dstAddr',
      width: 160,
      ellipsis: true,
      render: (addr: string) => (
        <span className="font-mono text-[12px] text-gray-600 dark:text-gray-400">{addr || '-'}</span>
      ),
    },
    {
      title: '封包函数',
      dataIndex: 'packetFunction',
      key: 'packetFunction',
      width: 110,
      render: (func?: string) => (
        <Tag className="m-0 font-mono text-[11px] bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 border-none">{func || '-'}</Tag>
      ),
    },
    {
      title: '大小',
      dataIndex: 'size',
      key: 'size',
      width: 90,
      sorter: (a: Packet, b: Packet) => a.size - b.size,
      render: (size: number) => (
        <span className="font-medium text-gray-900 dark:text-gray-100 text-[12px]">{formatSize(size)}</span>
      ),
    },
    {
      title: '套接字',
      dataIndex: 'socket',
      key: 'socket',
      width: 100,
      render: (socket?: number) => (
        <span className="font-mono text-[11px] text-gray-500 dark:text-gray-400">{socket ? `0x${socket.toString(16)}` : '-'}</span>
      ),
    },
    {
      title: '封包数据',
      dataIndex: 'packetData',
      key: 'packetData',
      width: 350,
      ellipsis: {
        showTitle: false,
      },
      render: (data: string | undefined, record: Packet) => {
        let displayData = '';
        if (data) {
          displayData = data;
        } else if (record.rawData) {
          displayData = Array.from(record.rawData.slice(0, 32))
            .map((b) => b.toString(16).padStart(2, '0'))
            .join(' ') + (record.rawData.length > 32 ? '...' : '');
        } else {
          return <span className="text-[#bfbfbf] italic">-</span>;
        }
        
        return (
          <Tooltip title={displayData}>
            <span className="flex items-center gap-1.5 font-mono text-[11px] text-gray-600 dark:text-gray-400 break-all">
              <DatabaseOutlined className="text-[12px] text-gray-500 dark:text-gray-500" />
              {displayData}
            </span>
          </Tooltip>
        );
      },
    },
  ];


  const isDark = document.documentElement.classList.contains('dark');

  useEffect(() => {
    const updateScrollY = () => {
      if (containerRef.current) {
        const containerHeight = containerRef.current.clientHeight;
        // 减去表头高度（约40px）和边框等
        const calculatedHeight = containerHeight - 50;
        setScrollY(calculatedHeight > 0 ? calculatedHeight : undefined);
      }
    };

    updateScrollY();
    window.addEventListener('resize', updateScrollY);
    return () => window.removeEventListener('resize', updateScrollY);
  }, []);

  return (
    <div ref={containerRef} className="h-full w-full flex flex-col overflow-hidden">
      <Table
        columns={columns}
        dataSource={filteredPackets}
        rowKey="id"
        scroll={{ x: 'max-content', y: scrollY }}
        pagination={false}
        size="small"
        bordered
        className="text-xs h-full"
        style={{ 
          height: '100%',
        }}
        onRow={(record) => ({
          onClick: () => {
            handleSelectPacket(record);
          },
          style: {
            cursor: 'pointer',
            backgroundColor: record.id === selectedPacketId 
              ? (isDark ? 'rgba(30, 58, 95, 0.9)' : 'rgba(230, 247, 255, 0.9)')
              : (record.direction === 'send' 
                  ? (isDark ? '#1e3a5f' : '#e6f7ff')
                  : (isDark ? '#1e3a2e' : '#f6ffed')),
          },
        })}
        locale={{
          emptyText: (
            <div className="h-full flex items-center justify-center text-gray-400 dark:text-gray-500 text-sm" style={{ height: scrollY ? `${scrollY}px` : '100%' }}>
              暂无数据
            </div>
          ),
        }}
        components={{
          header: {
            cell: (props: any) => (
              <th {...props} className={`${props.className} ${isDark ? 'bg-gray-800 text-gray-100 border-gray-700' : 'bg-gray-50 text-gray-800 border-gray-200'}`} />
            ),
          },
          body: {
            cell: (props: any) => (
              <td {...props} className={`${props.className} ${isDark ? 'bg-gray-900 text-gray-100 border-gray-700' : 'bg-white text-gray-900 border-gray-200'}`} />
            ),
          },
        }}
      />
    </div>
  );
};

