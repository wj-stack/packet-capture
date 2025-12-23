import React, { useState } from 'react';
import { Space, Select, Button, Checkbox, InputNumber } from 'antd';
import { PlayCircleOutlined, StopOutlined, ClearOutlined } from '@ant-design/icons';
import { useApp } from '../../contexts/AppContext';
import { captureApi } from '../../api';
import styles from './CaptureControls.module.css';


export const CaptureControls: React.FC = () => {
  const { state, dispatch } = useApp();
  const { captureStatus } = state;
  const [filterBySize, setFilterBySize] = useState(false);
  const [minSize, setMinSize] = useState<number | null>(null);
  const [maxSize, setMaxSize] = useState<number | null>(null);

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

  const isCapturing = captureStatus === 'capturing';

  return (
    <div className={styles.captureControls}>
      <Space size="middle" wrap>
        <Button
          type={captureStatus === 'capturing' ? 'default' : 'primary'}
          icon={<PlayCircleOutlined />}
          onClick={handleStartCapture}
          disabled={captureStatus === 'capturing'}
        >
          开始抓包
        </Button>
        <Button
          icon={<StopOutlined />}
          onClick={handleStopCapture}
          disabled={captureStatus === 'idle'}
        >
          结束抓包
        </Button>
        <Button icon={<ClearOutlined />} onClick={handleClearData}>
          清空列表
        </Button>

        <Checkbox
          checked={filterBySize}
          onChange={(e) => setFilterBySize(e.target.checked)}
          disabled={isCapturing}
        >
          过滤包长
        </Checkbox>
        {filterBySize && (
          <>
            <span className={styles.label}>最短:</span>
            <InputNumber
              value={minSize}
              onChange={setMinSize}
              min={0}
              style={{ width: 100 }}
              placeholder="最小长度"
              disabled={isCapturing}
            />
            <span className={styles.label}>最长:</span>
            <InputNumber
              value={maxSize}
              onChange={setMaxSize}
              min={0}
              style={{ width: 100 }}
              placeholder="最大长度"
              disabled={isCapturing}
            />
          </>
        )}
      </Space>
    </div>
  );
};

