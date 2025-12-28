import React, { useState, useEffect } from 'react';
import {
  List,
  Button,
  Modal,
  Form,
  Input,
  message,
  Popconfirm,
  Empty,
  Card,
  Typography,
  Space,
  Tag,
} from 'antd';
import {
  SaveOutlined,
  DeleteOutlined,
  FileTextOutlined,
  ClockCircleOutlined,
} from '@ant-design/icons';
import type { SavedQuery, CreateSavedQueryRequest } from '../../types';
import {
  listSavedQueries,
  createSavedQuery,
  deleteSavedQuery,
} from '../../services/savedQuery';

const { Text } = Typography;
const { TextArea } = Input;

interface SavedQueriesProps {
  domainId: string | null;
  connectionId: string | null;
  onQuerySelect: (queryText: string) => void;
  currentQuery?: string;
}

const SavedQueries: React.FC<SavedQueriesProps> = ({
  domainId,
  connectionId,
  onQuerySelect,
  currentQuery,
}) => {
  const [savedQueries, setSavedQueries] = useState<SavedQuery[]>([]);
  const [loading, setLoading] = useState(false);
  const [saveModalVisible, setSaveModalVisible] = useState(false);
  const [form] = Form.useForm();

  // Load saved queries when domain changes
  useEffect(() => {
    if (domainId) {
      loadSavedQueries();
    } else {
      setSavedQueries([]);
    }
  }, [domainId]);

  const loadSavedQueries = async () => {
    if (!domainId) return;

    setLoading(true);
    try {
      const queries = await listSavedQueries(domainId);
      setSavedQueries(queries);
    } catch (error) {
      console.error('Failed to load saved queries:', error);
      message.error('加载保存的查询失败');
    } finally {
      setLoading(false);
    }
  };

  const handleSaveQuery = async (values: { name: string; description?: string }) => {
    if (!domainId || !connectionId || !currentQuery) {
      message.error('请先选择域和连接，并输入查询');
      return;
    }

    try {
      const request: CreateSavedQueryRequest = {
        connection_id: connectionId,
        name: values.name,
        query_text: currentQuery,
        description: values.description,
      };

      await createSavedQuery(domainId, request);
      message.success('查询保存成功');
      setSaveModalVisible(false);
      form.resetFields();
      loadSavedQueries();
    } catch (error) {
      console.error('Failed to save query:', error);
      message.error('保存查询失败');
    }
  };

  const handleDeleteQuery = async (queryId: string) => {
    if (!domainId) return;

    try {
      await deleteSavedQuery(domainId, queryId);
      message.success('查询删除成功');
      loadSavedQueries();
    } catch (error) {
      console.error('Failed to delete query:', error);
      message.error('删除查询失败');
    }
  };

  const handleLoadQuery = (query: SavedQuery) => {
    onQuerySelect(query.query_text);
    message.success(`已加载查询: ${query.name}`);
  };

  const openSaveModal = () => {
    if (!currentQuery || currentQuery.trim() === '') {
      message.warning('请先输入要保存的查询');
      return;
    }
    if (!connectionId) {
      message.warning('请先选择一个数据库连接');
      return;
    }
    setSaveModalVisible(true);
  };

  if (!domainId) {
    return (
      <Card style={{ height: '100%' }}>
        <Empty
          description="请先选择一个工作域"
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      </Card>
    );
  }

  return (
    <Card
      title={
        <Space>
          <FileTextOutlined />
          <span>保存的查询</span>
          <Tag color="blue">{savedQueries.length}</Tag>
        </Space>
      }
      extra={
        <Button
          type="primary"
          icon={<SaveOutlined />}
          onClick={openSaveModal}
          disabled={!connectionId || !currentQuery}
        >
          保存当前查询
        </Button>
      }
      style={{ height: '100%' }}
    >
      <List
        loading={loading}
        dataSource={savedQueries}
        locale={{
          emptyText: (
            <Empty
              description="暂无保存的查询"
              image={Empty.PRESENTED_IMAGE_SIMPLE}
            />
          ),
        }}
        renderItem={(query) => (
          <List.Item
            key={query.id}
            actions={[
              <Popconfirm
                key="delete"
                title="确定要删除这个查询吗?"
                onConfirm={() => handleDeleteQuery(query.id)}
                okText="删除"
                cancelText="取消"
                okButtonProps={{ danger: true }}
              >
                <Button
                  type="text"
                  danger
                  size="small"
                  icon={<DeleteOutlined />}
                >
                  删除
                </Button>
              </Popconfirm>,
            ]}
            style={{ cursor: 'pointer' }}
            onClick={() => handleLoadQuery(query)}
          >
            <List.Item.Meta
              title={
                <Space>
                  <Text strong>{query.name}</Text>
                  {query.description && (
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      ({query.description})
                    </Text>
                  )}
                </Space>
              }
              description={
                <Space direction="vertical" size={4} style={{ width: '100%' }}>
                  <Text
                    code
                    ellipsis
                    style={{ fontSize: 12, display: 'block', maxWidth: 400 }}
                  >
                    {query.query_text}
                  </Text>
                  <Space size={16} style={{ fontSize: 12, color: '#8c8c8c' }}>
                    <span>
                      <ClockCircleOutlined /> {new Date(query.created_at).toLocaleString('zh-CN')}
                    </span>
                  </Space>
                </Space>
              }
            />
          </List.Item>
        )}
      />

      {/* Save Query Modal */}
      <Modal
        title="保存查询"
        open={saveModalVisible}
        onCancel={() => {
          setSaveModalVisible(false);
          form.resetFields();
        }}
        onOk={() => form.submit()}
        okText="保存"
        cancelText="取消"
      >
        <Form form={form} layout="vertical" onFinish={handleSaveQuery}>
          <Form.Item
            name="name"
            label="查询名称"
            rules={[
              { required: true, message: '请输入查询名称' },
              { max: 100, message: '名称不能超过100个字符' },
            ]}
          >
            <Input placeholder="例如: 获取用户列表" />
          </Form.Item>

          <Form.Item
            name="description"
            label="描述 (可选)"
            rules={[{ max: 500, message: '描述不能超过500个字符' }]}
          >
            <TextArea
              rows={3}
              placeholder="简要描述这个查询的用途..."
            />
          </Form.Item>

          <Form.Item label="查询预览">
            <TextArea
              value={currentQuery}
              rows={6}
              disabled
              style={{ fontFamily: 'monospace', fontSize: 12 }}
            />
          </Form.Item>
        </Form>
      </Modal>
    </Card>
  );
};

export default SavedQueries;
