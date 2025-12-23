import { ConfigProvider } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import { AppProvider } from './contexts/AppContext';
import { MainLayout } from './components/MainLayout';
import './App.css';

function App() {
  return (
    <ConfigProvider locale={zhCN}>
      <AppProvider>
        <MainLayout />
      </AppProvider>
    </ConfigProvider>
  );
}

export default App;
