import React, { useState } from 'react';
import { Layout, Button } from 'antd';
import { MenuFoldOutlined, MenuUnfoldOutlined } from '@ant-design/icons';

const { Sider, Content } = Layout;

interface AWSThreePanelLayoutProps {
  /**
   * Left panel content (navigation, filters, etc.)
   */
  leftPanel?: React.ReactNode;

  /**
   * Main content area
   */
  children: React.ReactNode;

  /**
   * Right panel content (details, info, etc.)
   * Optional - only shown when provided
   */
  rightPanel?: React.ReactNode;

  /**
   * Width of left panel (default: 280px)
   */
  leftPanelWidth?: number;

  /**
   * Width of right panel (default: 320px)
   */
  rightPanelWidth?: number;

  /**
   * Whether left panel is collapsible (default: true)
   */
  leftCollapsible?: boolean;

  /**
   * Whether right panel is collapsible (default: true)
   */
  rightCollapsible?: boolean;
}

/**
 * AWS-style three-panel layout
 * Mimics AWS Console's left navigation + center content + right details pattern
 */
export const AWSThreePanelLayout: React.FC<AWSThreePanelLayoutProps> = ({
  leftPanel,
  children,
  rightPanel,
  leftPanelWidth = 280,
  rightPanelWidth = 320,
  leftCollapsible = true,
  rightCollapsible = true,
}) => {
  const [leftCollapsed, setLeftCollapsed] = useState(false);
  const [rightCollapsed, setRightCollapsed] = useState(false);

  return (
    <Layout style={{ minHeight: '100%', background: 'var(--aws-light-gray)' }}>
      {/* Left Panel */}
      {leftPanel && (
        <Sider
          width={leftPanelWidth}
          collapsedWidth={0}
          collapsed={leftCollapsed}
          collapsible={leftCollapsible}
          trigger={null}
          style={{
            background: '#FFFFFF',
            borderRight: '1px solid var(--aws-border-gray)',
            overflow: 'auto',
            height: '100vh',
            position: 'sticky',
            top: 0,
            left: 0,
          }}
        >
          {leftCollapsible && (
            <div
              style={{
                padding: '12px',
                borderBottom: '1px solid var(--aws-border-gray)',
                textAlign: 'right',
              }}
            >
              <Button
                type="text"
                icon={leftCollapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
                onClick={() => setLeftCollapsed(!leftCollapsed)}
                size="small"
              />
            </div>
          )}
          <div style={{ padding: '16px' }}>{leftPanel}</div>
        </Sider>
      )}

      {/* Main Content */}
      <Layout style={{ background: 'var(--aws-light-gray)' }}>
        <Content
          style={{
            padding: '0',
            minHeight: '100vh',
            background: 'var(--aws-light-gray)',
          }}
        >
          {children}
        </Content>
      </Layout>

      {/* Right Panel */}
      {rightPanel && (
        <Sider
          width={rightPanelWidth}
          collapsedWidth={0}
          collapsed={rightCollapsed}
          collapsible={rightCollapsible}
          trigger={null}
          style={{
            background: '#FFFFFF',
            borderLeft: '1px solid var(--aws-border-gray)',
            overflow: 'auto',
            height: '100vh',
            position: 'sticky',
            top: 0,
            right: 0,
          }}
        >
          {rightCollapsible && (
            <div
              style={{
                padding: '12px',
                borderBottom: '1px solid var(--aws-border-gray)',
                textAlign: 'left',
              }}
            >
              <Button
                type="text"
                icon={rightCollapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
                onClick={() => setRightCollapsed(!rightCollapsed)}
                size="small"
              />
            </div>
          )}
          <div style={{ padding: '16px' }}>{rightPanel}</div>
        </Sider>
      )}
    </Layout>
  );
};

/**
 * AWS-style split panel layout (two panels only)
 * Useful for simple left nav + content or content + right details layouts
 */
interface AWSSplitPanelLayoutProps {
  /**
   * Side panel content (can be left or right)
   */
  sidePanel: React.ReactNode;

  /**
   * Main content
   */
  children: React.ReactNode;

  /**
   * Position of side panel (default: 'left')
   */
  sidePanelPosition?: 'left' | 'right';

  /**
   * Width of side panel (default: 280px)
   */
  sidePanelWidth?: number;

  /**
   * Whether side panel is collapsible (default: true)
   */
  collapsible?: boolean;
}

export const AWSSplitPanelLayout: React.FC<AWSSplitPanelLayoutProps> = ({
  sidePanel,
  children,
  sidePanelPosition = 'left',
  sidePanelWidth = 280,
  collapsible = true,
}) => {
  const [collapsed, setCollapsed] = useState(false);

  const siderComponent = (
    <Sider
      width={sidePanelWidth}
      collapsedWidth={0}
      collapsed={collapsed}
      collapsible={collapsible}
      trigger={null}
      style={{
        background: '#FFFFFF',
        borderRight: sidePanelPosition === 'left' ? '1px solid var(--aws-border-gray)' : 'none',
        borderLeft: sidePanelPosition === 'right' ? '1px solid var(--aws-border-gray)' : 'none',
        overflow: 'auto',
        height: '100vh',
        position: 'sticky',
        top: 0,
      }}
    >
      {collapsible && (
        <div
          style={{
            padding: '12px',
            borderBottom: '1px solid var(--aws-border-gray)',
            textAlign: sidePanelPosition === 'left' ? 'right' : 'left',
          }}
        >
          <Button
            type="text"
            icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
            onClick={() => setCollapsed(!collapsed)}
            size="small"
          />
        </div>
      )}
      <div style={{ padding: '16px' }}>{sidePanel}</div>
    </Sider>
  );

  return (
    <Layout style={{ minHeight: '100%', background: 'var(--aws-light-gray)' }}>
      {sidePanelPosition === 'left' && siderComponent}

      <Layout style={{ background: 'var(--aws-light-gray)' }}>
        <Content
          style={{
            padding: '0',
            minHeight: '100vh',
            background: 'var(--aws-light-gray)',
          }}
        >
          {children}
        </Content>
      </Layout>

      {sidePanelPosition === 'right' && siderComponent}
    </Layout>
  );
};
