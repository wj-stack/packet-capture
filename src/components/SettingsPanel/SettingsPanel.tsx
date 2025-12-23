import React from 'react';
import { Drawer, Form, InputNumber, Switch, Select, Button, Space, Input } from 'antd';
import { useApp } from '../../contexts/AppContext';
import { settingsApi } from '../../api';
import type { AppSettings, Theme, TimestampMode, ExportFormat } from '../../types';

const { Option } = Select;

export const SettingsPanel: React.FC = () => {
  const { state, dispatch } = useApp();
  const { ui, settings } = state;
  const [form] = Form.useForm();

  React.useEffect(() => {
    form.setFieldsValue(settings);
  }, [settings, form]);

  const handleClose = () => {
    dispatch({ type: 'TOGGLE_SETTINGS' });
  };

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      await settingsApi.updateSettings(values);
      dispatch({ type: 'UPDATE_SETTINGS', payload: values });
      handleClose();
    } catch (error) {
      console.error('保存设置失败:', error);
    }
  };

  const handleReset = async () => {
    try {
      await settingsApi.resetSettings();
      // TODO: 重新加载设置
      handleClose();
    } catch (error) {
      console.error('重置设置失败:', error);
    }
  };

  return (
    <Drawer
      title="设置"
      open={ui.settingsVisible}
      onClose={handleClose}
      width={600}
      extra={
        <Space>
          <Button onClick={handleReset}>重置</Button>
          <Button onClick={handleClose}>取消</Button>
          <Button type="primary" onClick={handleSave}>
            保存
          </Button>
        </Space>
      }
    >
      <Form form={form} layout="vertical">
        <h3>抓包设置</h3>
        <Form.Item label="最大数据包数量" name={['capture', 'maxPackets']}>
          <InputNumber style={{ width: '100%' }} min={100} max={1000000} />
        </Form.Item>
        <Form.Item name={['capture', 'autoCleanup']} valuePropName="checked">
          <Switch checkedChildren="启用" unCheckedChildren="禁用" />
          <span style={{ marginLeft: 8 }}>自动清理</span>
        </Form.Item>
        <Form.Item label="清理间隔 (分钟)" name={['capture', 'cleanupInterval']}>
          <InputNumber style={{ width: '100%' }} min={1} />
        </Form.Item>
        <Form.Item label="缓冲区大小 (字节)" name={['capture', 'bufferSize']}>
          <InputNumber style={{ width: '100%' }} min={1024} />
        </Form.Item>

        <h3>显示设置</h3>
        <Form.Item label="主题" name={['display', 'theme']}>
          <Select>
            <Option value="light">浅色</Option>
            <Option value="dark">深色</Option>
            <Option value="auto">自动</Option>
          </Select>
        </Form.Item>
        <Form.Item label="字体大小" name={['display', 'fontSize']}>
          <InputNumber style={{ width: '100%' }} min={10} max={20} />
        </Form.Item>
        <Form.Item label="时间戳显示" name={['display', 'showTimestamp']}>
          <Select>
            <Option value="absolute">绝对时间</Option>
            <Option value="relative">相对时间</Option>
          </Select>
        </Form.Item>
        <Form.Item label="日期格式" name={['display', 'dateFormat']}>
          <Input placeholder="YYYY-MM-DD HH:mm:ss" />
        </Form.Item>

        <h3>过滤设置</h3>
        <Form.Item name={['filter', 'rememberFilters']} valuePropName="checked">
          <Switch checkedChildren="启用" unCheckedChildren="禁用" />
          <span style={{ marginLeft: 8 }}>记住过滤器</span>
        </Form.Item>
        <Form.Item name={['filter', 'autoApply']} valuePropName="checked">
          <Switch checkedChildren="启用" unCheckedChildren="禁用" />
          <span style={{ marginLeft: 8 }}>自动应用过滤器</span>
        </Form.Item>

        <h3>导出设置</h3>
        <Form.Item label="默认格式" name={['export', 'defaultFormat']}>
          <Select>
            <Option value="json">JSON</Option>
            <Option value="har">HAR</Option>
            <Option value="pcap">PCAP</Option>
            <Option value="csv">CSV</Option>
          </Select>
        </Form.Item>
        <Form.Item label="默认路径" name={['export', 'defaultPath']}>
          <Input placeholder="导出文件默认保存路径" />
        </Form.Item>
        <Form.Item name={['export', 'includeRawData']} valuePropName="checked">
          <Switch checkedChildren="启用" unCheckedChildren="禁用" />
          <span style={{ marginLeft: 8 }}>包含原始数据</span>
        </Form.Item>
        <Form.Item name={['export', 'compress']} valuePropName="checked">
          <Switch checkedChildren="启用" unCheckedChildren="禁用" />
          <span style={{ marginLeft: 8 }}>压缩导出文件</span>
        </Form.Item>

        <h3>高级设置</h3>
        <Form.Item label="日志级别" name={['advanced', 'logLevel']}>
          <Select>
            <Option value="debug">Debug</Option>
            <Option value="info">Info</Option>
            <Option value="warn">Warn</Option>
            <Option value="error">Error</Option>
          </Select>
        </Form.Item>
        <Form.Item name={['advanced', 'enablePerformanceMonitor']} valuePropName="checked">
          <Switch checkedChildren="启用" unCheckedChildren="禁用" />
          <span style={{ marginLeft: 8 }}>启用性能监控</span>
        </Form.Item>
        <Form.Item name={['advanced', 'enableDevTools']} valuePropName="checked">
          <Switch checkedChildren="启用" unCheckedChildren="禁用" />
          <span style={{ marginLeft: 8 }}>启用开发者工具</span>
        </Form.Item>
      </Form>
    </Drawer>
  );
};

