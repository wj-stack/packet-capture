import React, { useState } from 'react';
import { Table, Input, Button, Select, Checkbox, Space } from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';
import type { PacketFeatureRule, PacketFunction } from '../../types';
import styles from './PacketInterceptPanel.module.css';

const { Option } = Select;
const { TextArea } = Input;

const PACKET_FUNCTIONS: PacketFunction[] = [
  'Send',
  'Recv',
  'SendTo',
  'RecvFrom',
  'WSASend',
  'WSARecv',
  'WSASendTo',
  'WSARecvFrom',
];

export const PacketInterceptPanel: React.FC = () => {
  const [rules, setRules] = useState<PacketFeatureRule[]>([]);
  const [feature, setFeature] = useState('');
  const [replacement, setReplacement] = useState('');
  const [selectedType, setSelectedType] = useState<PacketFunction>('Send');
  const [onlyRecord, setOnlyRecord] = useState(false);
  const [filter, setFilter] = useState(false);
  const [replace, setReplace] = useState(false);
  const [interceptLog, setInterceptLog] = useState<string[]>([]);

  const handleAddFeature = () => {
    if (!feature.trim()) {
      return;
    }

    const newRule: PacketFeatureRule = {
      id: Date.now().toString(),
      type: selectedType,
      feature: feature.trim(),
      replacement: replacement.trim() || undefined,
      onlyRecord,
      filter,
      replace,
      matchCount: 0,
    };

    setRules([...rules, newRule]);
    setFeature('');
    setReplacement('');
    setInterceptLog([
      ...interceptLog,
      `[${new Date().toLocaleTimeString()}] 添加特征规则: ${newRule.type} - ${newRule.feature}`,
    ]);
  };

  const handleDeleteRule = (id: string) => {
    setRules(rules.filter((r) => r.id !== id));
    setInterceptLog([
      ...interceptLog,
      `[${new Date().toLocaleTimeString()}] 删除特征规则: ${id}`,
    ]);
  };

  const columns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
      width: 50,
      render: (_: string, __: PacketFeatureRule, index: number) => index + 1,
    },
    {
      title: '类型',
      dataIndex: 'type',
      key: 'type',
      width: 90,
    },
    {
      title: '封包特征',
      dataIndex: 'feature',
      key: 'feature',
      width: 200,
      ellipsis: {
        showTitle: false,
      },
      render: (text: string) => (
        <span title={text} style={{ fontFamily: 'monospace', fontSize: '12px' }}>
          {text}
        </span>
      ),
    },
    {
      title: '匹配次数',
      dataIndex: 'matchCount',
      key: 'matchCount',
      width: 90,
      render: (count: number) => count || 0,
    },
    {
      title: '操作',
      key: 'action',
      width: 70,
      render: (_: any, record: PacketFeatureRule) => (
        <Button
          type="link"
          danger
          size="small"
          icon={<DeleteOutlined />}
          onClick={() => handleDeleteRule(record.id)}
        >
          删除
        </Button>
      ),
    },
  ];

  return (
    <div className={styles.packetInterceptPanel}>
      <div className={styles.title}>封包拦截-过滤-网截-修改-记录</div>

      <div className={styles.section}>
        <Table
          columns={columns}
          dataSource={rules}
          rowKey="id"
          size="small"
          pagination={false}
          scroll={{ y: 200 }}
        />
      </div>

      <div className={styles.section}>
        <div className={styles.inputGroup}>
          <span className={styles.label}>待过滤特征:</span>
          <Input
            value={feature}
            onChange={(e) => setFeature(e.target.value)}
            placeholder="输入封包特征（十六进制或文本）"
          />
        </div>
        <div className={styles.inputGroup}>
          <span className={styles.label}>替换为特征:</span>
          <Input
            value={replacement}
            onChange={(e) => setReplacement(e.target.value)}
            placeholder="输入替换特征（可选）"
          />
        </div>
      </div>

      <div className={styles.section}>
        <Space wrap>
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={handleAddFeature}
            disabled={!feature.trim()}
          >
            加入特征
          </Button>
          <Select
            value={selectedType}
            onChange={setSelectedType}
            style={{ width: 120 }}
          >
            {PACKET_FUNCTIONS.map((func) => (
              <Option key={func} value={func}>
                {func}
              </Option>
            ))}
          </Select>
          <Checkbox checked={onlyRecord} onChange={(e) => setOnlyRecord(e.target.checked)}>
            仅记录次数
          </Checkbox>
          <Checkbox checked={filter} onChange={(e) => setFilter(e.target.checked)}>
            过滤
          </Checkbox>
          <Checkbox checked={replace} onChange={(e) => setReplace(e.target.checked)}>
            替换
          </Checkbox>
        </Space>
      </div>

      <div className={styles.section}>
        <div className={styles.label}>封包特征拦截状态-</div>
        <TextArea
          value={interceptLog.join('\n')}
          readOnly
          rows={8}
          className={styles.logArea}
          placeholder="拦截日志将显示在这里..."
        />
      </div>
    </div>
  );
};

