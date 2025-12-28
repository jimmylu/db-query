import React, { useState, useEffect } from 'react';
import {
  Card,
  Table,
  Button,
  Modal,
  Form,
  Input,
  message,
  Space,
  Popconfirm,
  Tag,
  Typography,
} from 'antd';
import {
  FolderOutlined,
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  DatabaseOutlined,
} from '@ant-design/icons';
import { domainService } from '../../services/domain';
import { DomainResponse, CreateDomainRequest, UpdateDomainRequest } from '../../types';

const { Text } = Typography;
const { TextArea } = Input;

interface DomainManagerProps {
  onDomainSelect?: (domain: DomainResponse) => void;
}

export const DomainManager: React.FC<DomainManagerProps> = ({ onDomainSelect }) => {
  const [domains, setDomains] = useState<DomainResponse[]>([]);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [editingDomain, setEditingDomain] = useState<DomainResponse | null>(null);
  const [form] = Form.useForm();

  useEffect(() => {
    loadDomains();
  }, []);

  const loadDomains = async () => {
    setLoading(true);
    try {
      const data = await domainService.listDomains();
      setDomains(data);
    } catch (error: any) {
      message.error(`加载Domain失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = () => {
    setEditingDomain(null);
    form.resetFields();
    setModalVisible(true);
  };

  const handleEdit = (domain: DomainResponse) => {
    setEditingDomain(domain);
    form.setFieldsValue({
      name: domain.name,
      description: domain.description,
    });
    setModalVisible(true);
  };

  const handleDelete = async (id: string) => {
    try {
      await domainService.deleteDomain(id);
      message.success('Domain删除成功');
      loadDomains();
    } catch (error: any) {
      message.error(`删除失败: ${error.response?.data?.error || error.message}`);
    }
  };

  const handleSubmit = async (values: CreateDomainRequest | UpdateDomainRequest) => {
    try {
      if (editingDomain) {
        // Update existing domain
        await domainService.updateDomain(editingDomain.id, values as UpdateDomainRequest);
        message.success('Domain更新成功');
      } else {
        // Create new domain
        await domainService.createDomain(values as CreateDomainRequest);
        message.success('Domain创建成功');
      }
      setModalVisible(false);
      form.resetFields();
      loadDomains();
    } catch (error: any) {
      const errorMsg = error.response?.data?.error || error.message;
      message.error(`操作失败: ${errorMsg}`);
    }
  };

  const columns = [
    {
      title: 'Domain名称',
      dataIndex: 'name',
      key: 'name',
      render: (text: string) => (
        <Space>
          <FolderOutlined />
          <Text strong>{text}</Text>
        </Space>
      ),
    },
    {
      title: '描述',
      dataIndex: 'description',
      key: 'description',
      render: (text: string) => text || <Text type="secondary">-</Text>,
    },
    {
      title: '连接数',
      dataIndex: 'connection_count',
      key: 'connection_count',
      width: 100,
      render: (count: number) => (
        <Tag icon={<DatabaseOutlined />} color="blue">
          {count}
        </Tag>
      ),
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      key: 'created_at',
      width: 180,
      render: (date: string) => new Date(date).toLocaleString('zh-CN'),
    },
    {
      title: '操作',
      key: 'actions',
      width: 150,
      render: (_: any, record: DomainResponse) => {
        const isDefault = record.id === 'default-domain-id';
        return (
          <Space>
            <Button
              type="link"
              size="small"
              icon={<EditOutlined />}
              onClick={() => handleEdit(record)}
              disabled={isDefault}
            >
              编辑
            </Button>
            <Popconfirm
              title="确定要删除这个Domain吗？"
              description="删除Domain会同时删除其下所有连接！"
              onConfirm={() => handleDelete(record.id)}
              okText="确定"
              cancelText="取消"
              disabled={isDefault}
            >
              <Button
                type="link"
                size="small"
                danger
                icon={<DeleteOutlined />}
                disabled={isDefault}
              >
                删除
              </Button>
            </Popconfirm>
          </Space>
        );
      },
    },
  ];

  return (
    <>
      <Card
        title={
          <Space>
            <FolderOutlined />
            <span>Domain管理</span>
          </Space>
        }
        extra={
          <Button type="primary" icon={<PlusOutlined />} onClick={handleCreate}>
            新建Domain
          </Button>
        }
      >
        <Table
          columns={columns}
          dataSource={domains}
          loading={loading}
          rowKey="id"
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showTotal: (total) => `共 ${total} 个Domain`,
          }}
          onRow={(record) => ({
            onClick: () => onDomainSelect?.(record),
            style: { cursor: onDomainSelect ? 'pointer' : 'default' },
          })}
        />
      </Card>

      <Modal
        title={editingDomain ? '编辑Domain' : '新建Domain'}
        open={modalVisible}
        onCancel={() => {
          setModalVisible(false);
          form.resetFields();
        }}
        onOk={() => form.submit()}
        okText="确定"
        cancelText="取消"
      >
        <Form form={form} layout="vertical" onFinish={handleSubmit}>
          <Form.Item
            label="Domain名称"
            name="name"
            rules={[
              { required: true, message: '请输入Domain名称' },
              { max: 50, message: '名称不能超过50个字符' },
              {
                pattern: /^[a-zA-Z0-9\s\-_]+$/,
                message: '只能包含字母、数字、空格、连字符和下划线',
              },
            ]}
          >
            <Input placeholder="例如: Production, Development, Analytics" />
          </Form.Item>

          <Form.Item
            label="描述"
            name="description"
            rules={[{ max: 500, message: '描述不能超过500个字符' }]}
          >
            <TextArea
              rows={3}
              placeholder="可选，描述这个Domain的用途"
              maxLength={500}
              showCount
            />
          </Form.Item>
        </Form>
      </Modal>
    </>
  );
};
