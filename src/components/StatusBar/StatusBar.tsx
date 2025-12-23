import React from 'react';
import { Space, Tag } from 'antd';
import { useApp } from '../../contexts/AppContext';
import { formatFileSize } from '../../utils/formatters';
import styles from './StatusBar.module.css';

export const StatusBar: React.FC = () => {
  const { state } = useApp();
  const { statistics, captureStatus } = state;

  const getStatusColor = () => {
    switch (captureStatus) {
      case 'capturing':
        return 'success';
      case 'stopping':
        return 'warning';
      default:
        return 'default';
    }
  };

  const getStatusText = () => {
    switch (captureStatus) {
      case 'capturing':
        return '运行中';
      case 'stopping':
        return '停止中';
      default:
        return '空闲';
    }
  };

  return (
    <div className={styles.statusBar}>
      <Space split={<span className={styles.separator}>|</span>}>
        <span>
          已捕获: <strong>{statistics.totalPackets}</strong>
        </span>
        <span>
          过滤后: <strong>{statistics.filteredPackets}</strong>
        </span>
        <span>
          内存使用: <strong>{formatFileSize(statistics.memoryUsage)}</strong>
        </span>
        <span>
          状态: <Tag color={getStatusColor()}>{getStatusText()}</Tag>
        </span>
        {statistics.activeReplaceRules > 0 && (
          <span>
            活跃替换规则: <strong>{statistics.activeReplaceRules}</strong>
          </span>
        )}
      </Space>
    </div>
  );
};

