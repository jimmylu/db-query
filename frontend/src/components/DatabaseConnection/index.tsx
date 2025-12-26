import React, { useState } from 'react';
import { Form, Input, Button, Select, message, Card, Space } from 'antd';
import { DatabaseOutlined, CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons';
import { connectionService } from '../../services/connection';
import { CreateConnectionRequest, DatabaseConnection } from '../../types';

const { TextArea } = Input;

interface DatabaseConnectionProps {
  onConnectionCreated?: (connection: DatabaseConnection) => void;
}

export const DatabaseConnectionComponent: React.FC<DatabaseConnectionProps> = ({ onConnectionCreated }) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [selectedDbType, setSelectedDbType] = useState<string>('postgresql');

  const getPlaceholder = (dbType: string) => {
    switch (dbType) {
      case 'mysql':
        return 'mysql://user:password@localhost:3306/dbname';
      case 'postgresql':
      default:
        return 'postgresql://user:password@localhost:5432/dbname';
    }
  };

  const handleSubmit = async (values: CreateConnectionRequest) => {
    setLoading(true);
    setConnectionStatus('idle');

    try {
      const result = await connectionService.createConnection({
        name: values.name,
        connection_url: values.connection_url,
        database_type: values.database_type || 'postgresql',
      });

      message.success('数据库连接成功！');
      setConnectionStatus('success');
      form.resetFields();
      
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

  return (
    <Card
      title={
        <Space>
          <DatabaseOutlined />
          <span>数据库连接</span>
        </Space>
      }
      style={{ marginBottom: 24 }}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        initialValues={{
          database_type: 'postgresql',
        }}
      >
        <Form.Item
          label="连接名称"
          name="name"
          tooltip="可选，用于标识此连接"
        >
          <Input placeholder="例如: 生产数据库" />
        </Form.Item>

        <Form.Item
          label="数据库类型"
          name="database_type"
        >
          <Select onChange={(value) => setSelectedDbType(value)}>
            <Select.Option value="postgresql">PostgreSQL</Select.Option>
            <Select.Option value="mysql">MySQL</Select.Option>
          </Select>
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
                // Basic URL validation - specific format validation is done on backend
                if (!value.includes('://')) {
                  return Promise.reject(new Error('URL 格式不正确，必须包含协议（如 postgresql://, mysql://）'));
                }
                return Promise.resolve();
              },
            },
          ]}
        >
          <TextArea
            rows={2}
            placeholder={getPlaceholder(selectedDbType)}
          />
        </Form.Item>

        <Form.Item>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              loading={loading}
              icon={<DatabaseOutlined />}
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
    </Card>
  );
};

