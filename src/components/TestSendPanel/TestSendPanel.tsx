import React from 'react';
import { Drawer, Tabs, Form, Input, Select, Button, Space, InputNumber } from 'antd';
import { useApp } from '../../contexts/AppContext';
import { testSendApi } from '../../api';
import type { TestPacket } from '../../types';

const { Option } = Select;
const { TextArea } = Input;

const HTTP_METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'];

export const TestSendPanel: React.FC = () => {
  const { state, dispatch } = useApp();
  const { ui } = state;
  const [form] = Form.useForm();

  const handleClose = () => {
    dispatch({ type: 'TOGGLE_TEST_SEND_PANEL' });
  };

  const handleSend = async () => {
    try {
      const values = await form.validateFields();
      const packet: TestPacket = {
        protocol: values.protocol,
        method: values.method,
        url: values.url,
        headers: values.headers ? JSON.parse(values.headers) : undefined,
        body: values.body,
        targetProcessId: values.targetProcessId,
        delay: values.delay,
      };
      await testSendApi.sendTestPacket(packet);
      // TODO: 显示成功消息
      handleClose();
    } catch (error) {
      console.error('发送失败:', error);
      // TODO: 显示错误消息
    }
  };

  const httpTab = (
    <Form form={form} layout="vertical">
      <Form.Item label="协议" name="protocol" initialValue="HTTP" rules={[{ required: true }]}>
        <Select>
          <Option value="HTTP">HTTP</Option>
          <Option value="HTTPS">HTTPS</Option>
        </Select>
      </Form.Item>
      <Form.Item label="方法" name="method" initialValue="GET" rules={[{ required: true }]}>
        <Select>
          {HTTP_METHODS.map((m) => (
            <Option key={m} value={m}>
              {m}
            </Option>
          ))}
        </Select>
      </Form.Item>
      <Form.Item label="URL" name="url" rules={[{ required: true }]}>
        <Input placeholder="https://example.com/api/endpoint" />
      </Form.Item>
      <Form.Item label="请求头 (JSON格式)" name="headers">
        <TextArea rows={4} placeholder='{"Content-Type": "application/json"}' />
      </Form.Item>
      <Form.Item label="请求体" name="body">
        <TextArea rows={6} placeholder="请求体内容" />
      </Form.Item>
      <Form.Item label="目标进程ID (可选)" name="targetProcessId">
        <InputNumber style={{ width: '100%' }} placeholder="留空则发送到默认进程" />
      </Form.Item>
      <Form.Item label="延迟发送 (毫秒)" name="delay">
        <InputNumber style={{ width: '100%' }} min={0} placeholder="0" />
      </Form.Item>
    </Form>
  );

  const tcpUdpTab = (
    <Form form={form} layout="vertical">
      <Form.Item label="协议" name="protocol" initialValue="TCP" rules={[{ required: true }]}>
        <Select>
          <Option value="TCP">TCP</Option>
          <Option value="UDP">UDP</Option>
        </Select>
      </Form.Item>
      <Form.Item label="目标地址" name="dstAddr" rules={[{ required: true }]}>
        <Input placeholder="192.168.1.1:8080" />
      </Form.Item>
      <Form.Item label="源地址 (可选)" name="srcAddr">
        <Input placeholder="192.168.1.2:12345" />
      </Form.Item>
      <Form.Item label="数据内容 (十六进制或文本)" name="body">
        <TextArea rows={6} placeholder="输入数据内容" />
      </Form.Item>
      <Form.Item label="目标进程ID (可选)" name="targetProcessId">
        <InputNumber style={{ width: '100%' }} placeholder="留空则发送到默认进程" />
      </Form.Item>
      <Form.Item label="延迟发送 (毫秒)" name="delay">
        <InputNumber style={{ width: '100%' }} min={0} placeholder="0" />
      </Form.Item>
    </Form>
  );

  const tabItems = [
    {
      key: 'http',
      label: 'HTTP/HTTPS请求',
      children: httpTab,
    },
    {
      key: 'tcp',
      label: 'TCP/UDP数据包',
      children: tcpUdpTab,
    },
  ];

  return (
    <Drawer
      title="测试发包"
      open={ui.testSendPanelVisible}
      onClose={handleClose}
      width={600}
      extra={
        <Space>
          <Button onClick={handleClose}>取消</Button>
          <Button type="primary" onClick={handleSend}>
            发送
          </Button>
        </Space>
      }
    >
      <Tabs items={tabItems} />
    </Drawer>
  );
};

