import React from 'react';
import { Button, Space } from 'antd';
import {
  PlayCircleOutlined,
  StopOutlined,
  ClearOutlined,
  SendOutlined,
  ExportOutlined,
  SettingOutlined,
  SunOutlined,
  MoonOutlined,
} from '@ant-design/icons';
import { useApp } from '../../contexts/AppContext';
import { useTheme } from '../../contexts/ThemeContext';
import { captureApi } from '../../api';

export const TopToolbar: React.FC = () => {
  const { state, dispatch } = useApp();
  const { theme, toggleTheme } = useTheme();
  const { captureStatus } = state;

  const handleStartCapture = async () => {
    try {
      await captureApi.startCapture();
      dispatch({ type: 'START_CAPTURE' });
    } catch (error) {
      console.error('开始抓包失败:', error);
    }
  };

  const handleStopCapture = async () => {
    try {
      await captureApi.stopCapture();
      dispatch({ type: 'STOP_CAPTURE' });
    } catch (error) {
      console.error('停止抓包失败:', error);
    }
  };

  const handleClearData = () => {
    dispatch({ type: 'CLEAR_PACKETS' });
  };

  const handleTestSend = () => {
    dispatch({ type: 'TOGGLE_TEST_SEND_PANEL' });
  };

  const handleExport = () => {
    // TODO: 实现导出功能
    console.log('导出数据包');
  };

  const handleSettings = () => {
    dispatch({ type: 'TOGGLE_SETTINGS' });
  };

  return (
    <div className="py-1.5 px-2 sm:px-3 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 min-h-10 flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-2 shadow-sm">
      <Space size="small" wrap className="flex-1">
        <Button
          type={captureStatus === 'capturing' ? 'primary' : 'default'}
          icon={<PlayCircleOutlined />}
          onClick={handleStartCapture}
          disabled={captureStatus === 'capturing'}
          className="shadow-sm"
          size="small"
        >
          <span className="hidden sm:inline">开始抓包</span>
          <span className="sm:hidden">开始</span>
        </Button>
        <Button
          type={captureStatus === 'stopping' ? 'primary' : 'default'}
          icon={<StopOutlined />}
          onClick={handleStopCapture}
          disabled={captureStatus === 'idle'}
          className="shadow-sm"
          size="small"
        >
          <span className="hidden sm:inline">停止抓包</span>
          <span className="sm:hidden">停止</span>
        </Button>
        <Button
          danger
          icon={<ClearOutlined />}
          onClick={handleClearData}
          disabled={state.packets.length === 0}
          className="shadow-sm"
          size="small"
        >
          <span className="hidden sm:inline">清空数据</span>
          <span className="sm:hidden">清空</span>
        </Button>
        <Button
          type="primary"
          className="bg-green-500 hover:bg-green-600 border-green-500 hover:border-green-600 shadow-sm"
          icon={<SendOutlined />}
          onClick={handleTestSend}
          size="small"
        >
          <span className="hidden md:inline">测试发包</span>
          <span className="md:hidden">测试</span>
        </Button>
        <Button 
          icon={<ExportOutlined />} 
          onClick={handleExport} 
          className="shadow-sm hidden sm:inline-flex"
          size="small"
        >
          导出
        </Button>
        <Button 
          icon={<SettingOutlined />} 
          onClick={handleSettings} 
          className="shadow-sm"
          size="small"
        >
          <span className="hidden sm:inline">设置</span>
        </Button>
      </Space>
      <Button
        icon={theme === 'light' ? <SunOutlined /> : <MoonOutlined />}
        onClick={toggleTheme}
        className="shadow-sm flex-shrink-0"
        size="small"
        title={theme === 'light' ? '切换到深色主题' : '切换到浅色主题'}
      />
    </div>
  );
};

