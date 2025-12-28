import React, { useState, useEffect } from 'react';
import { Form, Input, Button, message, Card, Space, Row, Col, Typography, Alert, Tag } from 'antd';
import { CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons';
import { connectionService } from '../../services/connection';
import { domainService } from '../../services/domain';
import { CreateConnectionRequest, DatabaseConnection, DomainResponse } from '../../types';
import { DatabaseLogo } from '../../assets/database-logos';

const { TextArea } = Input;
const { Title, Text } = Typography;

interface DatabaseConnectionProps {
  onConnectionCreated?: (connection: DatabaseConnection) => void;
}

type DatabaseType = 'postgresql' | 'mysql' | 'doris' | 'druid';

interface DatabaseOption {
  type: DatabaseType;
  name: string;
  description: string;
  placeholder: string;
  urlExample: string;
}

const DATABASE_OPTIONS: DatabaseOption[] = [
  {
    type: 'postgresql',
    name: 'PostgreSQL',
    description: '开源关系型数据库',
    placeholder: 'postgresql://user:password@localhost:5432/dbname',
    urlExample: 'postgresql://user:password@localhost:5432/dbname',
  },
  {
    type: 'mysql',
    name: 'MySQL',
    description: '流行的开源关系型数据库',
    placeholder: 'mysql://user:password@localhost:3306/dbname',
    urlExample: 'mysql://user:password@localhost:3306/dbname',
  },
  {
    type: 'doris',
    name: 'Apache Doris',
    description: '高性能实时分析数据库',
    placeholder: 'mysql://user:password@localhost:9030/dbname',
    urlExample: 'mysql://user:password@fe_host:9030/dbname',
  },
  {
    type: 'druid',
    name: 'Apache Druid',
    description: '实时 OLAP 分析引擎',
    placeholder: 'druid://localhost:8888',
    urlExample: 'druid://broker_host:8888 或 http://broker_host:8888',
  },
];

export const DatabaseConnectionComponent: React.FC<DatabaseConnectionProps> = ({ onConnectionCreated }) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [selectedDbType, setSelectedDbType] = useState<DatabaseType | null>(null);
  const [currentDomainId, setCurrentDomainId] = useState<string | null>(null);
  const [currentDomain, setCurrentDomain] = useState<DomainResponse | null>(null);

  useEffect(() => {
    // Listen for global domain changes from CustomHeader
    const handleDomainChanged = (event: CustomEvent) => {
      const domainId = event.detail.domainId;
      const domain = event.detail.domain;

      setCurrentDomainId(domainId);
      if (domain) {
        setCurrentDomain(domain);
      } else if (domainId) {
        // Fetch domain details if not provided
        domainService.getDomain(domainId)
          .then(setCurrentDomain)
          .catch(console.error);
      }
    };

    window.addEventListener('domainChanged', handleDomainChanged as EventListener);

    // Load initial domain from localStorage
    const storedDomainId = localStorage.getItem('selectedDomainId');
    if (storedDomainId) {
      setCurrentDomainId(storedDomainId);
      domainService.getDomain(storedDomainId)
        .then(setCurrentDomain)
        .catch(console.error);
    }

    return () => {
      window.removeEventListener('domainChanged', handleDomainChanged as EventListener);
    };
  }, []);

  const handleDatabaseSelect = (dbType: DatabaseType) => {
    setSelectedDbType(dbType);
    setConnectionStatus('idle');
    form.setFieldsValue({ database_type: dbType });
  };

  const handleSubmit = async (values: CreateConnectionRequest) => {
    if (!selectedDbType) {
      message.error('请先选择数据库类型');
      return;
    }

    setLoading(true);
    setConnectionStatus('idle');

    try {
      const result = await connectionService.createConnection({
        name: values.name,
        connection_url: values.connection_url,
        database_type: selectedDbType,
        domain_id: currentDomainId || undefined,
      });

      message.success('数据库连接成功！');
      setConnectionStatus('success');
      form.resetFields();
      setSelectedDbType(null);

      if (onConnectionCreated) {
        onConnectionCreated(result.connection);
      }
    } catch (error: any) {
      const errorMessage = error.response?.data?.error?.message || error.message || '连接失败';
      message.error(`连接失败: ${errorMessage}`);
      setConnectionStatus('error');
    } finally {
      setLoading(false);
    }
  };

  const selectedDbOption = DATABASE_OPTIONS.find(opt => opt.type === selectedDbType);

  return (
    <div style={{ padding: '24px' }}>
      {/* Domain Display */}
      {currentDomain && (
        <Alert
          message={
            <Space>
              <Text strong>当前工作域:</Text>
              <Tag color="blue">{currentDomain.name}</Tag>
              {currentDomain.description && (
                <Text type="secondary">({currentDomain.description})</Text>
              )}
            </Space>
          }
          type="info"
          showIcon
          style={{ marginBottom: 24 }}
        />
      )}

      <Card
        title={<Title level={4} style={{ margin: 0 }}>新建数据库连接</Title>}
        style={{ marginBottom: 24 }}
      >
        {/* Database Type Selection */}
        {!selectedDbType && (
          <>
            <Title level={5} style={{ marginBottom: 16 }}>选择数据源类型</Title>
            <Row gutter={[16, 16]}>
              {DATABASE_OPTIONS.map((option) => (
                <Col key={option.type} xs={12} sm={12} md={6} lg={6}>
                  <Card
                    hoverable
                    onClick={() => handleDatabaseSelect(option.type)}
                    style={{
                      textAlign: 'center',
                      cursor: 'pointer',
                      height: '100%',
                      border: selectedDbType === option.type ? '2px solid #1890ff' : '1px solid #d9d9d9',
                    }}
                    bodyStyle={{ padding: '24px 16px' }}
                  >
                    <div style={{ marginBottom: 12 }}>
                      <DatabaseLogo type={option.type} size={64} />
                    </div>
                    <Title level={5} style={{ margin: '8px 0 4px' }}>{option.name}</Title>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      {option.description}
                    </Text>
                  </Card>
                </Col>
              ))}
            </Row>
          </>
        )}

        {/* Connection Form */}
        {selectedDbType && selectedDbOption && (
          <>
            <div style={{ marginBottom: 16, display: 'flex', alignItems: 'center', gap: 12 }}>
              <DatabaseLogo type={selectedDbType} size={40} />
              <div>
                <Title level={5} style={{ margin: 0 }}>{selectedDbOption.name}</Title>
                <Text type="secondary" style={{ fontSize: 12 }}>{selectedDbOption.description}</Text>
              </div>
              <Button
                type="link"
                onClick={() => setSelectedDbType(null)}
                style={{ marginLeft: 'auto' }}
              >
                重新选择
              </Button>
            </div>

            <Form
              form={form}
              layout="vertical"
              onFinish={handleSubmit}
              initialValues={{
                database_type: selectedDbType,
              }}
            >
              <Form.Item name="database_type" hidden>
                <Input />
              </Form.Item>

              <Form.Item
                label="连接名称"
                name="name"
                tooltip="可选，用于标识此连接"
              >
                <Input placeholder="例如: 生产数据库" size="large" />
              </Form.Item>

              <Form.Item
                label="连接 URL"
                name="connection_url"
                rules={[
                  { required: true, message: '请输入数据库连接 URL' },
                  {
                    validator: (_, value) => {
                      if (!value) {
                        return Promise.reject(new Error('请输入数据库连接 URL'));
                      }
                      if (!value.includes('://')) {
                        return Promise.reject(new Error('URL 格式不正确，必须包含协议'));
                      }
                      return Promise.resolve();
                    },
                  },
                ]}
                extra={
                  <Text type="secondary" style={{ fontSize: 12 }}>
                    示例: {selectedDbOption.urlExample}
                  </Text>
                }
              >
                <TextArea
                  rows={3}
                  placeholder={selectedDbOption.placeholder}
                  size="large"
                />
              </Form.Item>

              <Form.Item>
                <Space>
                  <Button
                    type="primary"
                    htmlType="submit"
                    loading={loading}
                    size="large"
                  >
                    连接
                  </Button>
                  {connectionStatus === 'success' && (
                    <span style={{ color: '#52c41a' }}>
                      <CheckCircleOutlined /> 连接成功
                    </span>
                  )}
                  {connectionStatus === 'error' && (
                    <span style={{ color: '#ff4d4f' }}>
                      <CloseCircleOutlined /> 连接失败
                    </span>
                  )}
                </Space>
              </Form.Item>
            </Form>
          </>
        )}
      </Card>
    </div>
  );
};
