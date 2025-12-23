import React from 'react';
import { Layout } from 'antd';
import { PacketList } from '../PacketList';
import { CaptureControls } from '../CaptureControls';
import { PacketFunctionFilter } from '../PacketFunctionFilter';
import { PacketPreview } from '../PacketPreview';
import { PacketInterceptPanel } from '../PacketInterceptPanel';
import { StatusBar } from '../StatusBar';
import { TestSendPanel } from '../TestSendPanel';
import { SettingsPanel } from '../SettingsPanel';
import styles from './MainLayout.module.css';

const { Content, Sider } = Layout;

export const MainLayout: React.FC = () => {
  return (
    <div className={styles.layout}>
      <Layout className={styles.layoutInner}>
        <Content className={styles.content}>
          <div className={styles.leftPanel}>
            <div className={styles.packetListSection}>
              <div className={styles.sectionTitle}>封包数据</div>
              <PacketList />
            </div>
            <CaptureControls />
            <PacketFunctionFilter />
            <PacketPreview />
          </div>
          <Sider width={600} className={styles.rightPanel}>
            <PacketInterceptPanel />
          </Sider>
        </Content>
      </Layout>
      <div className={styles.statusBarWrapper}>
        <StatusBar />
      </div>
      <TestSendPanel />
      <SettingsPanel />
    </div>
  );
};

