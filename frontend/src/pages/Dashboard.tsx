import React, { useState, useEffect } from 'react';
import { Table, Tag, Button, Popconfirm, Space, Drawer, Typography, Descriptions, Modal, message } from 'antd';
import { DatabaseOutlined, DeleteOutlined, CheckCircleOutlined, CloseCircleOutlined, ClockCircleOutlined, PlusOutlined, EyeOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { DatabaseConnectionComponent } from '../components/DatabaseConnection';
import { MetadataViewer } from '../components/MetadataViewer';
import { DatabaseConnection } from '../types';
import { connectionService } from '../services/connection';
import { AWSContainer, AWSPageHeader } from '../components/AWSContainer';

const { Title } = Typography;

export const Dashboard: React.FC = () => {
  // Initialize selectedDomainId from localStorage synchronously
  const initialDomainId = localStorage.getItem('selectedDomainId');

  const [connections, setConnections] = useState<DatabaseConnection[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedDomainId, setSelectedDomainId] = useState<string | null>(initialDomainId);
  const [selectedConnection, setSelectedConnection] = useState<DatabaseConnection | null>(null);
  const [detailDrawerVisible, setDetailDrawerVisible] = useState(false);
  const [newConnectionModalVisible, setNewConnectionModalVisible] = useState(false);

  useEffect(() => {
    loadConnections();

    // Listen for global domain changes from CustomHeader
    const handleDomainChanged = (event: CustomEvent) => {
      const newDomainId = event.detail.domainId;
      console.log('Dashboard received domain change:', newDomainId);
      setSelectedDomainId(newDomainId);
    };

    window.addEventListener('domainChanged', handleDomainChanged as EventListener);

    // Note: selectedDomainId is already initialized from localStorage in useState
    console.log('Dashboard initialized with domain from localStorage:', initialDomainId);

    return () => {
      window.removeEventListener('domainChanged', handleDomainChanged as EventListener);
    };
  }, []);

  const loadConnections = async () => {
    setLoading(true);
    try {
      const conns = await connectionService.listConnections();
      setConnections(conns);
    } catch (error: any) {
      console.error('Failed to load connections:', error);
      message.error('加载数据库连接失败');
    } finally {
      setLoading(false);
    }
  };

  const handleConnectionCreated = () => {
    setNewConnectionModalVisible(false);
    loadConnections();
    message.success('数据库连接创建成功');
  };

  const handleDeleteConnection = async (id: string) => {
    try {
      await connectionService.deleteConnection(id);
      message.success('连接已删除');
      if (selectedConnection?.id === id) {
        setSelectedConnection(null);
        setDetailDrawerVisible(false);
      }
      loadConnections();
    } catch (error: any) {
      const errorMessage = error.response?.data?.error?.message || error.message || '删除失败';
      message.error(`删除失败: ${errorMessage}`);
    }
  };

  const handleViewDetails = (connection: DatabaseConnection) => {
    setSelectedConnection(connection);
    setDetailDrawerVisible(true);
  };

  // Filter connections by selected domain
  const filteredConnections = selectedDomainId
    ? connections.filter(conn => {
        console.log(`Filtering connection ${conn.id}: domain_id=${conn.domain_id}, selected=${selectedDomainId}, match=${conn.domain_id === selectedDomainId}`);
        return conn.domain_id === selectedDomainId;
      })
    : connections;

  console.log('Dashboard state:', {
    selectedDomainId,
    totalConnections: connections.length,
    filteredConnections: filteredConnections.length,
    connections: connections.map(c => ({ id: c.id, name: c.name, domain_id: c.domain_id }))
  });

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

  const getDatabaseTypeTag = (dbType: string) => {
    const typeColors: Record<string, string> = {
      postgresql: 'blue',
      mysql: 'orange',
      doris: 'purple',
      druid: 'cyan',
    };
    return <Tag color={typeColors[dbType] || 'default'}>{dbType.toUpperCase()}</Tag>;
  };

  const columns: ColumnsType<DatabaseConnection> = [
    {
      title: '连接名称',
      dataIndex: 'name',
      key: 'name',
      render: (name, record) => (
        <Space>
          <DatabaseOutlined style={{ color: '#1890ff' }} />
          <span>{name || record.connection_url.split('@')[1] || '未命名连接'}</span>
        </Space>
      ),
    },
    {
      title: '数据库类型',
      dataIndex: 'database_type',
      key: 'database_type',
      width: 120,
      render: (dbType) => getDatabaseTypeTag(dbType),
    },
    {
      title: '连接状态',
      dataIndex: 'status',
      key: 'status',
      width: 110,
      render: (status) => getStatusTag(status),
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      width: 180,
      render: (created_at) => created_at ? new Date(created_at).toLocaleString('zh-CN') : '-',
    },
    {
      title: '操作',
      key: 'actions',
      width: 180,
      render: (_, record) => (
        <Space>
          <Button
            type="link"
            icon={<EyeOutlined />}
            onClick={() => handleViewDetails(record)}
          >
            查看详情
          </Button>
          <Popconfirm
            title="确定要删除这个连接吗？"
            onConfirm={() => handleDeleteConnection(record.id)}
            okText="确定"
            cancelText="取消"
          >
            <Button
              type="link"
              danger
              icon={<DeleteOutlined />}
            >
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div style={{ padding: '24px', background: 'var(--aws-light-gray)' }}>
      <AWSPageHeader
        title="数据集列表"
        description="管理和查看数据库连接"
        actions={
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => setNewConnectionModalVisible(true)}
          >
            新建连接
          </Button>
        }
      />

      <AWSContainer>
        <Table
          columns={columns}
          dataSource={filteredConnections}
          loading={loading}
          rowKey="id"
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showTotal: (total) => `共 ${total} 个数据集`,
          }}
          locale={{
            emptyText: selectedDomainId ? '此 Domain 暂无数据集' : '暂无数据集，请点击"新建连接"添加',
          }}
        />
      </AWSContainer>

      {/* Detail Drawer */}
      <Drawer
        title={
          <Space>
            <DatabaseOutlined />
            <span>数据集详情</span>
          </Space>
        }
        placement="right"
        size="large"
        onClose={() => setDetailDrawerVisible(false)}
        open={detailDrawerVisible}
      >
        {selectedConnection && (
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <Descriptions bordered column={1}>
              <Descriptions.Item label="连接名称">
                {selectedConnection.name || '未命名'}
              </Descriptions.Item>
              <Descriptions.Item label="数据库类型">
                {getDatabaseTypeTag(selectedConnection.database_type)}
              </Descriptions.Item>
              <Descriptions.Item label="连接状态">
                {getStatusTag(selectedConnection.status)}
              </Descriptions.Item>
              <Descriptions.Item label="连接 URL">
                {selectedConnection.connection_url}
              </Descriptions.Item>
              <Descriptions.Item label="连接 ID">
                <code style={{ fontSize: '12px' }}>{selectedConnection.id}</code>
              </Descriptions.Item>
              <Descriptions.Item label="创建时间">
                {selectedConnection.created_at
                  ? new Date(selectedConnection.created_at).toLocaleString('zh-CN')
                  : '-'}
              </Descriptions.Item>
              <Descriptions.Item label="最后连接时间">
                {selectedConnection.last_connected_at
                  ? new Date(selectedConnection.last_connected_at).toLocaleString('zh-CN')
                  : '从未连接'}
              </Descriptions.Item>
            </Descriptions>

            <div>
              <Title level={5}>数据库元数据</Title>
              <MetadataViewer
                connectionId={selectedConnection.id}
                connection={selectedConnection}
              />
            </div>
          </Space>
        )}
      </Drawer>

      {/* New Connection Modal */}
      <Modal
        title={
          <Space>
            <DatabaseOutlined />
            <span>新建数据库连接</span>
          </Space>
        }
        open={newConnectionModalVisible}
        onCancel={() => setNewConnectionModalVisible(false)}
        footer={null}
        width={600}
      >
        <DatabaseConnectionComponent onConnectionCreated={handleConnectionCreated} />
      </Modal>
    </div>
  );
};
