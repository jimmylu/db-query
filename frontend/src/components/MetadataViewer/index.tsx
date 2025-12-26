import React, { useState, useEffect } from 'react';
import { Card, Table, Tag, Space, Button, Typography, Spin, Empty, Collapse } from 'antd';
import { DatabaseOutlined, ReloadOutlined, TableOutlined, EyeOutlined } from '@ant-design/icons';
import { metadataService } from '../../services/metadata';
import { DatabaseMetadata, Table as TableType, View as ViewType } from '../../types';

const { Title, Text } = Typography;
const { Panel } = Collapse;

interface MetadataViewerProps {
  connectionId: string | null;
}

export const MetadataViewer: React.FC<MetadataViewerProps> = ({ connectionId }) => {
  const [metadata, setMetadata] = useState<DatabaseMetadata | null>(null);
  const [loading, setLoading] = useState(false);
  const [cached, setCached] = useState(false);

  const loadMetadata = async (refresh: boolean = false) => {
    if (!connectionId) return;

    setLoading(true);
    try {
      const result = await metadataService.getMetadata(connectionId, refresh);
      setMetadata(result.metadata);
      setCached(result.cached);
    } catch (error: any) {
      console.error('Failed to load metadata:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (connectionId) {
      loadMetadata(false);
    } else {
      setMetadata(null);
    }
  }, [connectionId]);

  if (!connectionId) {
    return (
      <Card>
        <Empty description="请先连接数据库" />
      </Card>
    );
  }

  if (loading && !metadata) {
    return (
      <Card>
        <Spin size="large" style={{ display: 'block', textAlign: 'center', padding: '40px' }} />
      </Card>
    );
  }

  if (!metadata) {
    return (
      <Card>
        <Empty description="无法加载元数据" />
      </Card>
    );
  }

  const tableColumns = [
    {
      title: '列名',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: '数据类型',
      dataIndex: 'data_type',
      key: 'data_type',
    },
    {
      title: '可空',
      dataIndex: 'is_nullable',
      key: 'is_nullable',
      render: (nullable: boolean) => (
        <Tag color={nullable ? 'orange' : 'green'}>
          {nullable ? '是' : '否'}
        </Tag>
      ),
    },
    {
      title: '主键',
      dataIndex: 'is_primary_key',
      key: 'is_primary_key',
      render: (isPk: boolean) => isPk && <Tag color="blue">PK</Tag>,
    },
    {
      title: '外键',
      dataIndex: 'is_foreign_key',
      key: 'is_foreign_key',
      render: (isFk: boolean) => isFk && <Tag color="purple">FK</Tag>,
    },
  ];

  return (
    <Card
      title={
        <Space>
          <DatabaseOutlined />
          <span>数据库元数据</span>
          {cached && <Tag color="blue">缓存</Tag>}
        </Space>
      }
      extra={
        <Button
          icon={<ReloadOutlined />}
          onClick={() => loadMetadata(true)}
          loading={loading}
        >
          刷新
        </Button>
      }
    >
      <Space direction="vertical" style={{ width: '100%' }} size="large">
        {/* Schemas */}
        {metadata.schemas.length > 0 && (
          <div>
            <Title level={5}>Schemas ({metadata.schemas.length})</Title>
            <Space wrap>
              {metadata.schemas.map((schema) => (
                <Tag key={schema} color="cyan">
                  {schema}
                </Tag>
              ))}
            </Space>
          </div>
        )}

        {/* Tables */}
        {metadata.tables.length > 0 && (
          <div>
            <Title level={5}>
              <TableOutlined /> 表 ({metadata.tables.length})
            </Title>
            <Collapse>
              {metadata.tables.map((table: TableType) => (
                <Panel
                  key={`${table.schema || 'public'}.${table.name}`}
                  header={
                    <Space>
                      <span>{table.name}</span>
                      {table.schema && <Tag>{table.schema}</Tag>}
                      {table.row_count !== undefined && (
                        <Text type="secondary">({table.row_count} 行)</Text>
                      )}
                    </Space>
                  }
                >
                  <Table
                    columns={tableColumns}
                    dataSource={table.columns.map((col, idx) => ({ ...col, key: idx }))}
                    pagination={false}
                    size="small"
                  />
                </Panel>
              ))}
            </Collapse>
          </div>
        )}

        {/* Views */}
        {metadata.views.length > 0 && (
          <div>
            <Title level={5}>
              <EyeOutlined /> 视图 ({metadata.views.length})
            </Title>
            <Collapse>
              {metadata.views.map((view: ViewType) => (
                <Panel
                  key={`${view.schema || 'public'}.${view.name}`}
                  header={
                    <Space>
                      <span>{view.name}</span>
                      {view.schema && <Tag>{view.schema}</Tag>}
                    </Space>
                  }
                >
                  <Table
                    columns={tableColumns}
                    dataSource={view.columns.map((col, idx) => ({ ...col, key: idx }))}
                    pagination={false}
                    size="small"
                  />
                </Panel>
              ))}
            </Collapse>
          </div>
        )}

        {metadata.tables.length === 0 && metadata.views.length === 0 && (
          <Empty description="数据库中没有表或视图" />
        )}
      </Space>
    </Card>
  );
};

