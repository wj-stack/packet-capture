import React, { useEffect, useMemo } from 'react';
import { Space, Select, Input, InputNumber, Switch, Button } from 'antd';
import { CloseOutlined } from '@ant-design/icons';
import { useApp } from '../../contexts/AppContext';
import { usePacketFilter } from '../../utils/filter';
import type { Protocol } from '../../types';
import styles from './FilterBar.module.css';

const { Option } = Select;

const PROTOCOLS: Protocol[] = ['HTTP', 'HTTPS', 'TCP', 'UDP'];
const HTTP_METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'];
const DIRECTIONS = [
  { label: '全部', value: 'all' },
  { label: '发送', value: 'send' },
  { label: '接收', value: 'receive' },
];

export const FilterBar: React.FC = () => {
  const { state, dispatch } = useApp();
  const { filters, packets } = state;

  // 应用过滤
  const filteredPackets = usePacketFilter(packets, filters);

  useEffect(() => {
    dispatch({ type: 'SET_FILTERED_PACKETS', payload: filteredPackets });
  }, [filteredPackets, dispatch]);

  const handleProtocolChange = (values: Protocol[]) => {
    dispatch({
      type: 'SET_FILTERS',
      payload: { ...filters, protocols: values },
    });
  };

  const handleDirectionChange = (value: 'send' | 'receive' | 'all') => {
    dispatch({
      type: 'SET_FILTERS',
      payload: { ...filters, direction: value },
    });
  };

  const handleDomainChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    dispatch({
      type: 'SET_FILTERS',
      payload: { ...filters, domain: e.target.value || undefined },
    });
  };

  const handleMethodChange = (value: string) => {
    dispatch({
      type: 'SET_FILTERS',
      payload: { ...filters, method: value || undefined },
    });
  };

  const handleStatusCodeChange = (value: number | null) => {
    dispatch({
      type: 'SET_FILTERS',
      payload: { ...filters, statusCode: value || undefined },
    });
  };

  const handleMinSizeChange = (value: number | null) => {
    dispatch({
      type: 'SET_FILTERS',
      payload: { ...filters, minSize: value || undefined },
    });
  };

  const handleMaxSizeChange = (value: number | null) => {
    dispatch({
      type: 'SET_FILTERS',
      payload: { ...filters, maxSize: value || undefined },
    });
  };

  const handleKeywordChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    dispatch({
      type: 'SET_FILTERS',
      payload: { ...filters, keyword: e.target.value || undefined },
    });
  };

  const handleRegexToggle = (checked: boolean) => {
    dispatch({
      type: 'SET_FILTERS',
      payload: { ...filters, useRegex: checked },
    });
  };

  const handleClearFilters = () => {
    dispatch({
      type: 'SET_FILTERS',
      payload: {
        protocols: [],
        searchIn: ['url', 'header', 'body'],
      },
    });
  };

  const activeFilterCount = useMemo(() => {
    let count = 0;
    if (filters.protocols.length > 0) count++;
    if (filters.direction && filters.direction !== 'all') count++;
    if (filters.domain) count++;
    if (filters.method) count++;
    if (filters.statusCode) count++;
    if (filters.minSize || filters.maxSize) count++;
    if (filters.keyword) count++;
    return count;
  }, [filters]);

  return (
    <div className={styles.filterBar}>
      <Space wrap size="middle">
        <span className={styles.label}>协议:</span>
        <Select
          mode="multiple"
          placeholder="选择协议"
          value={filters.protocols}
          onChange={handleProtocolChange}
          style={{ width: 200 }}
        >
          {PROTOCOLS.map((p) => (
            <Option key={p} value={p}>
              {p}
            </Option>
          ))}
        </Select>

        <span className={styles.label}>方向:</span>
        <Select
          value={filters.direction || 'all'}
          onChange={handleDirectionChange}
          style={{ width: 120 }}
        >
          {DIRECTIONS.map((d) => (
            <Option key={d.value} value={d.value}>
              {d.label}
            </Option>
          ))}
        </Select>

        <span className={styles.label}>域名:</span>
        <Input
          placeholder="输入域名"
          value={filters.domain}
          onChange={handleDomainChange}
          style={{ width: 200 }}
          allowClear
        />

        <span className={styles.label}>方法:</span>
        <Select
          placeholder="HTTP方法"
          value={filters.method}
          onChange={handleMethodChange}
          style={{ width: 120 }}
          allowClear
        >
          {HTTP_METHODS.map((m) => (
            <Option key={m} value={m}>
              {m}
            </Option>
          ))}
        </Select>

        <span className={styles.label}>状态码:</span>
        <InputNumber
          placeholder="状态码"
          value={filters.statusCode}
          onChange={handleStatusCodeChange}
          style={{ width: 100 }}
          min={100}
          max={599}
        />

        <span className={styles.label}>大小:</span>
        <InputNumber
          placeholder="最小"
          value={filters.minSize}
          onChange={handleMinSizeChange}
          style={{ width: 100 }}
          min={0}
          addonAfter="B"
        />
        <span>-</span>
        <InputNumber
          placeholder="最大"
          value={filters.maxSize}
          onChange={handleMaxSizeChange}
          style={{ width: 100 }}
          min={0}
          addonAfter="B"
        />

        <span className={styles.label}>关键词:</span>
        <Input
          placeholder="搜索关键词"
          value={filters.keyword}
          onChange={handleKeywordChange}
          style={{ width: 200 }}
          allowClear
        />
        <Switch
          checkedChildren="正则"
          unCheckedChildren="普通"
          checked={filters.useRegex}
          onChange={handleRegexToggle}
        />

        {activeFilterCount > 0 && (
          <Button
            type="link"
            icon={<CloseOutlined />}
            onClick={handleClearFilters}
          >
            清空 ({activeFilterCount})
          </Button>
        )}
      </Space>
    </div>
  );
};

