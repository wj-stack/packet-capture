import React, { useState } from 'react';
import { Checkbox, Space } from 'antd';
import { useApp } from '../../contexts/AppContext';
import type { PacketFunction } from '../../types';

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

export const PacketFunctionFilter: React.FC = () => {
  const { state, dispatch } = useApp();
  const { filters, captureStatus } = state;
  const [selectedFunctions, setSelectedFunctions] = useState<PacketFunction[]>(
    filters.packetFunctions || []
  );

  const isCapturing = captureStatus === 'capturing';

  const handleFunctionChange = (func: PacketFunction, checked: boolean) => {
    let newFunctions: PacketFunction[];
    if (checked) {
      newFunctions = [...selectedFunctions, func];
    } else {
      newFunctions = selectedFunctions.filter((f) => f !== func);
    }
    setSelectedFunctions(newFunctions);
    dispatch({
      type: 'SET_FILTERS',
      payload: {
        ...filters,
        packetFunctions: newFunctions.length > 0 ? newFunctions : undefined,
      },
    });
  };

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      setSelectedFunctions(PACKET_FUNCTIONS);
      dispatch({
        type: 'SET_FILTERS',
        payload: {
          ...filters,
          packetFunctions: PACKET_FUNCTIONS,
        },
      });
    } else {
      setSelectedFunctions([]);
      dispatch({
        type: 'SET_FILTERS',
        payload: {
          ...filters,
          packetFunctions: undefined,
        },
      });
    }
  };

  const allSelected = selectedFunctions.length === PACKET_FUNCTIONS.length;

  return (
    <div className="px-2 sm:px-3 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded shadow-sm">
      <div className="font-semibold mb-1.5 text-xs text-gray-900 dark:text-gray-100">封包函数</div>
      <Space size="small" wrap className="w-full">
        <Checkbox
          checked={allSelected}
          indeterminate={selectedFunctions.length > 0 && !allSelected}
          onChange={(e) => handleSelectAll(e.target.checked)}
          disabled={isCapturing}
        >
          全选
        </Checkbox>
        {PACKET_FUNCTIONS.map((func) => (
          <Checkbox
            key={func}
            checked={selectedFunctions.includes(func)}
            onChange={(e) => handleFunctionChange(func, e.target.checked)}
            disabled={isCapturing}
          >
            {func}
          </Checkbox>
        ))}
      </Space>
    </div>
  );
};

