import React, { useState } from 'react';
import { Tabs } from 'antd';
import { DatabaseOutlined, ApartmentOutlined } from '@ant-design/icons';
import { QueryPage } from './QueryPage';
import { CrossDatabaseQueryPage } from './CrossDatabaseQueryPage';
import { AWSPageHeader } from '../components/AWSContainer';

const { TabPane } = Tabs;

export const DataExplorePage: React.FC = () => {
  const [activeTab, setActiveTab] = useState('single');

  return (
    <div style={{ padding: '24px', background: 'var(--aws-light-gray)' }}>
      <AWSPageHeader
        title="数据探索"
        description="执行 SQL 查询并探索数据库"
      />

      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
        size="large"
        type="card"
        style={{
          background: '#FFFFFF',
          padding: '16px 24px 0 24px',
          borderRadius: 'var(--aws-border-radius)',
          boxShadow: 'var(--aws-shadow-default)',
        }}
      >
        <TabPane
          tab={
            <span>
              <DatabaseOutlined />
              单库查询
            </span>
          }
          key="single"
        >
          <QueryPage />
        </TabPane>
        <TabPane
          tab={
            <span>
              <ApartmentOutlined />
              跨库查询
            </span>
          }
          key="cross"
        >
          <CrossDatabaseQueryPage />
        </TabPane>
      </Tabs>
    </div>
  );
};
