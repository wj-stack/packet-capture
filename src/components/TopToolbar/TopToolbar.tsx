import React from 'react';
import { Button, Space } from 'antd';
import {
  PlayCircleOutlined,
  StopOutlined,
  ClearOutlined,
  SendOutlined,
  ExportOutlined,
  SettingOutlined,
} from '@ant-design/icons';
import { useApp } from '../../contexts/AppContext';
import { captureApi } from '../../api';
import styles from './TopToolbar.module.css';

export const TopToolbar: React.FC = () => {
  const { state, dispatch } = useApp();
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
    <div className={styles.toolbar}>
      <Space>
        <Button
          type={captureStatus === 'capturing' ? 'primary' : 'default'}
          icon={<PlayCircleOutlined />}
          onClick={handleStartCapture}
          disabled={captureStatus === 'capturing'}
        >
          开始抓包
        </Button>
        <Button
          type={captureStatus === 'stopping' ? 'primary' : 'default'}
          icon={<StopOutlined />}
          onClick={handleStopCapture}
          disabled={captureStatus === 'idle'}
        >
          停止抓包
        </Button>
        <Button
          danger
          icon={<ClearOutlined />}
          onClick={handleClearData}
          disabled={state.packets.length === 0}
        >
          清空数据
        </Button>
        <Button
          type="primary"
          style={{ background: '#52c41a', borderColor: '#52c41a' }}
          icon={<SendOutlined />}
          onClick={handleTestSend}
        >
          测试发包
        </Button>
        <Button icon={<ExportOutlined />} onClick={handleExport}>
          导出
        </Button>
        <Button icon={<SettingOutlined />} onClick={handleSettings}>
          设置
        </Button>
      </Space>
    </div>
  );
};

