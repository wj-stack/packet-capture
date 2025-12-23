import React, { createContext, useContext, useReducer, ReactNode, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { AppState, AppAction, Packet } from '../types';
import { usePacketFilter } from '../utils/filter';

const initialState: AppState = {
  captureStatus: 'idle',
  packets: [],
  filteredPackets: [],
  selectedPacketId: undefined,
  packetsLoading: false,
  filters: {
    protocols: [],
    packetFunctions: undefined,
    searchIn: [],
  },
  filterPresets: [],
  testSendPanelVisible: false,
  testTemplates: [],
  replaceRules: [],
  replacePanelVisible: false,
  ui: {
    detailPanelVisible: false,
    detailPanelHeight: 300,
    settingsVisible: false,
    theme: 'light',
    testSendPanelVisible: false,
    replacePanelVisible: false,
  },
  settings: {
    capture: {
      maxPackets: 10000,
      autoCleanup: true,
      cleanupInterval: 60,
      bufferSize: 1024 * 1024,
      sampleRate: 1.0,
    },
    display: {
      theme: 'light',
      fontSize: 14,
      showTimestamp: 'absolute',
      dateFormat: 'YYYY-MM-DD HH:mm:ss',
      timezone: 'Asia/Shanghai',
    },
    filter: {
      defaultProtocols: [],
      rememberFilters: false,
      autoApply: true,
    },
    export: {
      defaultFormat: 'json',
      defaultPath: '',
      includeRawData: false,
      compress: false,
    },
    advanced: {
      logLevel: 'info',
      enablePerformanceMonitor: false,
      enableDevTools: false,
    },
  },
  statistics: {
    totalPackets: 0,
    filteredPackets: 0,
    memoryUsage: 0,
    networkSpeed: 0,
    activeReplaceRules: 0,
  },
};

function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'START_CAPTURE':
      return { ...state, captureStatus: 'capturing' };
    case 'STOP_CAPTURE':
      return { ...state, captureStatus: 'idle' };
    case 'SET_CAPTURE_STATUS':
      return { ...state, captureStatus: action.payload };
    case 'ADD_PACKET':
      return {
        ...state,
        packets: [...state.packets, action.payload],
        statistics: {
          ...state.statistics,
          totalPackets: state.statistics.totalPackets + 1,
        },
      };
    case 'ADD_PACKETS':
      return {
        ...state,
        packets: [...state.packets, ...action.payload],
        statistics: {
          ...state.statistics,
          totalPackets: state.statistics.totalPackets + action.payload.length,
        },
      };
    case 'CLEAR_PACKETS':
      return {
        ...state,
        packets: [],
        filteredPackets: [],
        selectedPacketId: undefined,
        statistics: {
          ...state.statistics,
          totalPackets: 0,
          filteredPackets: 0,
        },
      };
    case 'SELECT_PACKET':
      return {
        ...state,
        selectedPacketId: action.payload,
        ui: {
          ...state.ui,
          detailPanelVisible: action.payload !== undefined,
        },
      };
    case 'SET_FILTERS':
      return { ...state, filters: action.payload };
    case 'SET_FILTERED_PACKETS':
      return {
        ...state,
        filteredPackets: action.payload,
        statistics: {
          ...state.statistics,
          filteredPackets: action.payload.length,
        },
      };
    case 'SET_REPLACE_RULES':
      return {
        ...state,
        replaceRules: action.payload,
        statistics: {
          ...state.statistics,
          activeReplaceRules: action.payload.filter((r) => r.enabled).length,
        },
      };
    case 'ADD_REPLACE_RULE':
      return {
        ...state,
        replaceRules: [...state.replaceRules, action.payload],
        statistics: {
          ...state.statistics,
          activeReplaceRules:
            state.statistics.activeReplaceRules + (action.payload.enabled ? 1 : 0),
        },
      };
    case 'UPDATE_REPLACE_RULE':
      return {
        ...state,
        replaceRules: state.replaceRules.map((rule) =>
          rule.id === action.payload.id ? action.payload : rule
        ),
        statistics: {
          ...state.statistics,
          activeReplaceRules: state.replaceRules
            .map((r) => (r.id === action.payload.id ? action.payload : r))
            .filter((r) => r.enabled).length,
        },
      };
    case 'DELETE_REPLACE_RULE':
      const deletedRule = state.replaceRules.find((r) => r.id === action.payload);
      return {
        ...state,
        replaceRules: state.replaceRules.filter((r) => r.id !== action.payload),
        statistics: {
          ...state.statistics,
          activeReplaceRules:
            state.statistics.activeReplaceRules - (deletedRule?.enabled ? 1 : 0),
        },
      };
    case 'TOGGLE_TEST_SEND_PANEL':
      return {
        ...state,
        ui: {
          ...state.ui,
          testSendPanelVisible: !state.ui.testSendPanelVisible,
        },
      };
    case 'TOGGLE_REPLACE_PANEL':
      return {
        ...state,
        ui: {
          ...state.ui,
          replacePanelVisible: !state.ui.replacePanelVisible,
        },
      };
    case 'TOGGLE_SETTINGS':
      return {
        ...state,
        ui: {
          ...state.ui,
          settingsVisible: !state.ui.settingsVisible,
        },
      };
    case 'SET_DETAIL_PANEL_VISIBLE':
      return {
        ...state,
        ui: {
          ...state.ui,
          detailPanelVisible: action.payload,
        },
      };
    case 'SET_DETAIL_PANEL_HEIGHT':
      return {
        ...state,
        ui: {
          ...state.ui,
          detailPanelHeight: action.payload,
        },
      };
    case 'UPDATE_STATISTICS':
      return {
        ...state,
        statistics: {
          ...state.statistics,
          ...action.payload,
        },
      };
    case 'UPDATE_SETTINGS':
      return {
        ...state,
        settings: {
          ...state.settings,
          ...action.payload,
        },
      };
    default:
      return state;
  }
}

interface AppContextType {
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState);

  // 监听 Tauri 事件：封包捕获
  useEffect(() => {
    const setupEventListeners = async () => {
      // 监听封包捕获事件
      const unlistenPacket = await listen<Packet>('packet-captured', (event) => {
        dispatch({ type: 'ADD_PACKET', payload: event.payload });
      });

      // 清理函数
      return () => {
        unlistenPacket();
      };
    };

    const cleanup = setupEventListeners();
    
    return () => {
      cleanup.then((fn) => fn());
    };
  }, []);

  // 自动更新过滤后的封包列表
  useEffect(() => {
    const filtered = usePacketFilter(state.packets, state.filters);
    dispatch({ type: 'SET_FILTERED_PACKETS', payload: filtered });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state.packets, state.filters]);

  return <AppContext.Provider value={{ state, dispatch }}>{children}</AppContext.Provider>;
}

export function useApp() {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error('useApp must be used within AppProvider');
  }
  return context;
}

