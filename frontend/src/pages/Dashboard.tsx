import React, { useState, useEffect } from 'react';
import { Row, Col, List, Card, Tag, Button, Popconfirm, Space, Typography, Empty } from 'antd';
import { DatabaseOutlined, DeleteOutlined, CheckCircleOutlined, CloseCircleOutlined, ClockCircleOutlined } from '@ant-design/icons';
import { DatabaseConnectionComponent } from '../components/DatabaseConnection';
import { MetadataViewer } from '../components/MetadataViewer';
import { DatabaseConnection } from '../types';
import { connectionService } from '../services/connection';
import { message } from 'antd';

const { Title } = Typography;

export const Dashboard: React.FC = () => {
  const [selectedConnectionId, setSelectedConnectionId] = useState<string | null>(null);
  const [connections, setConnections] = useState<DatabaseConnection[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadConnections();
  }, []);

  const loadConnections = async () => {
    try {
      const conns = await connectionService.listConnections();
      setConnections(conns);
      if (conns.length > 0 && !selectedConnectionId) {
        setSelectedConnectionId(conns[0].id);
      }
    } catch (error: any) {
      console.error('Failed to load connections:', error);
    }
  };

  const handleConnectionCreated = (connection: DatabaseConnection) => {
    setSelectedConnectionId(connection.id);
    loadConnections();
  };

  const handleDeleteConnection = async (id: string) => {
    try {
      await connectionService.deleteConnection(id);
      message.success('连接已删除');
      if (selectedConnectionId === id) {
        setSelectedConnectionId(null);
      }
      loadConnections();
    } catch (error: any) {
      const errorMessage = error.response?.data?.error?.message || error.message || '删除失败';
      message.error(`删除失败: ${errorMessage}`);
    }
  };

  const getStatusTag = (status: string) => {
    switch (status) {
      case 'connected':
        return <Tag color="success" icon={<CheckCircleOutlined />}>已连接</Tag>;
      case 'error':
        return <Tag color="error" icon={<CloseCircleOutlined />}>错误</Tag>;
      case 'disconnected':
      default:
        return <Tag color="default" icon={<ClockCircleOutlined />}>未连接</Tag>;
    }
  };

  return (
    <div style={{ padding: '24px' }}>
      <Row gutter={[24, 24]}>
        <Col xs={24} lg={12}>
          <DatabaseConnectionComponent onConnectionCreated={handleConnectionCreated} />
          
          <Card
            title={
              <Space>
                <DatabaseOutlined />
                <span>已保存的连接</span>
              </Space>
            }
            style={{ marginTop: 24 }}
          >
            {connections.length === 0 ? (
              <Empty description="暂无保存的连接" />
            ) : (
              <List
                dataSource={connections}
                loading={loading}
                renderItem={(conn) => (
                  <List.Item
                    actions={[
                      <Popconfirm
                        title="确定要删除这个连接吗？"
                        onConfirm={() => handleDeleteConnection(conn.id)}
                        okText="确定"
                        cancelText="取消"
                      >
                        <Button
                          type="text"
                          danger
                          icon={<DeleteOutlined />}
                          size="small"
                        >
                          删除
                        </Button>
                      </Popconfirm>,
                    ]}
                  >
                    <List.Item.Meta
                      title={
                        <Space>
                          <span>{conn.name || conn.connection_url}</span>
                          {getStatusTag(conn.status)}
                        </Space>
                      }
                      description={
                        <div>
                          <div>{conn.connection_url}</div>
                          <div style={{ marginTop: 4, fontSize: '12px', color: '#999' }}>
                            {conn.created_at && new Date(conn.created_at).toLocaleString('zh-CN')}
                          </div>
                        </div>
                      }
                    />
                    <Button
                      type={selectedConnectionId === conn.id ? 'primary' : 'default'}
                      size="small"
                      onClick={() => setSelectedConnectionId(conn.id)}
                    >
                      {selectedConnectionId === conn.id ? '已选择' : '选择'}
                    </Button>
                  </List.Item>
                )}
              />
            )}
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <MetadataViewer connectionId={selectedConnectionId} />
        </Col>
      </Row>
    </div>
  );
};

