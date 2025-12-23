import React from 'react';
import { ConfigProvider, theme as antdTheme } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import { AppProvider } from './contexts/AppContext';
import { ThemeProvider, useTheme } from './contexts/ThemeContext';
import { MainLayout } from './components/MainLayout';
import './App.css';

function AppContent() {
  const { theme } = useTheme();
  
  React.useEffect(() => {
    // 应用全局样式
    document.documentElement.style.boxSizing = 'border-box';
    document.body.style.fontSize = '14px';
    document.body.style.fontFamily = '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif';
    const root = document.getElementById('root');
    if (root) {
      root.style.width = '100%';
      root.style.height = '100vh';
      root.style.overflow = 'hidden';
    }
  }, []);
  
  return (
    <ConfigProvider 
      locale={zhCN}
      theme={{
        algorithm: theme === 'dark' ? antdTheme.darkAlgorithm : antdTheme.defaultAlgorithm,
        token: {
          colorPrimary: theme === 'dark' ? '#1890ff' : '#1890ff',
        },
      }}
    >
      <AppProvider>
        <MainLayout />
      </AppProvider>
    </ConfigProvider>
  );
}

function App() {
  return (
    <ThemeProvider>
      <AppContent />
    </ThemeProvider>
  );
}

export default App;
