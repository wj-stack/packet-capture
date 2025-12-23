import React from 'react';
import { Space, Tag } from 'antd';
import { useApp } from '../../contexts/AppContext';
import { formatFileSize } from '../../utils/formatters';

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
    <div className="h-7 py-1 px-4 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 flex items-center text-xs text-gray-600 dark:text-gray-400 flex-shrink-0 shadow-sm">
      <Space split={<span className="mx-2 text-gray-300 dark:text-gray-600">|</span>}>
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

