import React from 'react';
import { Space, Typography } from 'antd';
import { DatabaseOutlined } from '@ant-design/icons';

const { Text } = Typography;

interface CustomTitleProps {
  collapsed?: boolean;
}

export const CustomTitle: React.FC<CustomTitleProps> = ({ collapsed }) => {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: collapsed ? 'center' : 'flex-start',
        padding: collapsed ? '0' : '0 16px',
        height: '48px',
        background: 'var(--aws-squid-ink)',
      }}
    >
      <Space size={collapsed ? 0 : 10}>
        {/* Logo Icon */}
        <div
          style={{
            width: collapsed ? 28 : 32,
            height: collapsed ? 28 : 32,
            borderRadius: '6px',
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            boxShadow: '0 1px 4px rgba(102, 126, 234, 0.3)',
          }}
        >
          <DatabaseOutlined
            style={{
              fontSize: collapsed ? 16 : 18,
              color: '#fff',
            }}
          />
        </div>

        {/* Title Text */}
        {!collapsed && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
            <Text
              strong
              style={{
                fontSize: '14px',
                color: '#FFFFFF',
                lineHeight: 1.2,
                whiteSpace: 'nowrap',
              }}
            >
              智慧数据探索工具
            </Text>
            <Text
              style={{
                fontSize: '10px',
                lineHeight: 1,
                whiteSpace: 'nowrap',
                color: '#AAB7B8',
              }}
            >
              Smart Data Explorer
            </Text>
          </div>
        )}
      </Space>
    </div>
  );
};
