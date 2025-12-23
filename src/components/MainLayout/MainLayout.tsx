import React, { useState, useEffect } from 'react';
import { Layout } from 'antd';
import { PacketList } from '../PacketList';
import { CaptureControls } from '../CaptureControls';
import { PacketFunctionFilter } from '../PacketFunctionFilter';
import { PacketPreview } from '../PacketPreview';
import { PacketInterceptPanel } from '../PacketInterceptPanel';
import { StatusBar } from '../StatusBar';

const { Content, Sider } = Layout;

export const MainLayout: React.FC = () => {
  const [isMobile, setIsMobile] = useState(false);
  const [siderCollapsed, setSiderCollapsed] = useState(false);

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 1024);
      if (window.innerWidth < 1024) {
        setSiderCollapsed(true);
      }
    };
    
    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  }, []);

  return (
    <div className="h-screen w-full flex flex-col overflow-hidden bg-gray-50 dark:bg-gray-900">
      <Layout className="flex-1 flex flex-col overflow-hidden min-h-0 bg-transparent">
        <Content className="flex-1 flex flex-col lg:flex-row overflow-hidden min-h-0">
          <div className="flex-1 flex flex-col p-1 sm:p-2 gap-1 sm:gap-2 overflow-hidden min-w-0">
            <div className="h-[600px] flex flex-col overflow-hidden">
              <div className="font-semibold text-xs mb-1 px-2 py-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded text-gray-900 dark:text-gray-100 flex-shrink-0">
                封包数据
              </div>
              <div className="flex-1 overflow-hidden border border-gray-200 dark:border-gray-700 border-t-0 bg-white dark:bg-gray-800 rounded-b min-h-0">
                <PacketList />
              </div>
            </div>
            <CaptureControls />
            <PacketFunctionFilter />
            <PacketPreview />
          </div>
          <Sider 
            width={isMobile ? 0 : 550}
            collapsedWidth={0}
            collapsed={siderCollapsed}
            collapsible
            onCollapse={setSiderCollapsed}
            breakpoint="lg"
            className="!bg-white dark:!bg-gray-800 border-l border-gray-200 dark:border-gray-700 overflow-y-auto hidden lg:block"
            style={{ minWidth: isMobile ? 0 : 550, maxWidth: isMobile ? 0 : 550 }}
          >
            <div className="h-full bg-white dark:bg-gray-800">
              <PacketInterceptPanel />
            </div>
          </Sider>
        </Content>
      </Layout>
      <div className="flex-shrink-0 w-full">
        <StatusBar />
      </div>
    </div>
  );
};

