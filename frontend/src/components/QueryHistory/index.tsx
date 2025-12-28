import React, { useState, useEffect } from 'react';
import {
  List,
  Card,
  Empty,
  Tag,
  Space,
  Typography,
  Tooltip,
  Badge,
  Button,
} from 'antd';
import {
  ClockCircleOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  RobotOutlined,
  UserOutlined,
  ThunderboltOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import type { QueryHistory } from '../../types';
import { listQueryHistory } from '../../services/savedQuery';

const { Text } = Typography;

interface QueryHistoryProps {
  domainId: string | null;
  onQuerySelect: (queryText: string) => void;
  limit?: number;
}

const QueryHistoryComponent: React.FC<QueryHistoryProps> = ({
  domainId,
  onQuerySelect,
  limit = 50,
}) => {
  const [history, setHistory] = useState<QueryHistory[]>([]);
  const [loading, setLoading] = useState(false);

  // Load query history when domain changes
  useEffect(() => {
    if (domainId) {
      loadQueryHistory();
    } else {
      setHistory([]);
    }
  }, [domainId]);

  const loadQueryHistory = async () => {
    if (!domainId) return;

    setLoading(true);
    try {
      const data = await listQueryHistory(domainId, limit);
      setHistory(data);
    } catch (error) {
      console.error('Failed to load query history:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleQueryClick = (queryText: string) => {
    onQuerySelect(queryText);
  };

  const formatExecutionTime = (ms: number): string => {
    if (ms < 1000) {
      return `${ms}ms`;
    }
    return `${(ms / 1000).toFixed(2)}s`;
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
          <ClockCircleOutlined />
          <span>查询历史</span>
          <Badge count={history.length} showZero overflowCount={999} />
        </Space>
      }
      extra={
        <Button
          icon={<ReloadOutlined />}
          onClick={loadQueryHistory}
          loading={loading}
          size="small"
        >
          刷新
        </Button>
      }
      style={{ height: '100%' }}
    >
      <List
        loading={loading}
        dataSource={history}
        locale={{
          emptyText: (
            <Empty
              description="暂无查询历史"
              image={Empty.PRESENTED_IMAGE_SIMPLE}
            />
          ),
        }}
        renderItem={(item) => (
          <List.Item
            key={item.id}
            style={{ cursor: 'pointer', transition: 'background 0.2s' }}
            onClick={() => handleQueryClick(item.query_text)}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = '#f5f5f5';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'transparent';
            }}
          >
            <List.Item.Meta
              avatar={
                item.status === 'success' ? (
                  <CheckCircleOutlined style={{ fontSize: 20, color: '#52c41a' }} />
                ) : (
                  <CloseCircleOutlined style={{ fontSize: 20, color: '#ff4d4f' }} />
                )
              }
              title={
                <Space size={8}>
                  {item.is_llm_generated ? (
                    <Tooltip title="AI生成的查询">
                      <Tag icon={<RobotOutlined />} color="purple">
                        AI
                      </Tag>
                    </Tooltip>
                  ) : (
                    <Tooltip title="手动编写的查询">
                      <Tag icon={<UserOutlined />} color="blue">
                        手动
                      </Tag>
                    </Tooltip>
                  )}
                  <Text
                    code
                    ellipsis
                    style={{ fontSize: 12, maxWidth: 400 }}
                  >
                    {item.query_text}
                  </Text>
                </Space>
              }
              description={
                <Space direction="vertical" size={4} style={{ width: '100%' }}>
                  {item.status === 'success' ? (
                    <Space size={16} style={{ fontSize: 12 }}>
                      <Text type="secondary">
                        <ThunderboltOutlined /> {formatExecutionTime(item.execution_time_ms)}
                      </Text>
                      <Text type="secondary">
                        {item.row_count} 行结果
                      </Text>
                    </Space>
                  ) : (
                    <Text type="danger" style={{ fontSize: 12 }}>
                      错误: {item.error_message}
                    </Text>
                  )}
                  <Text type="secondary" style={{ fontSize: 11 }}>
                    <ClockCircleOutlined /> {new Date(item.executed_at).toLocaleString('zh-CN')}
                  </Text>
                </Space>
              }
            />
          </List.Item>
        )}
        pagination={
          history.length > 20
            ? {
                pageSize: 20,
                size: 'small',
                showSizeChanger: false,
              }
            : false
        }
        style={{ maxHeight: 'calc(100vh - 300px)', overflow: 'auto' }}
      />
    </Card>
  );
};

export default QueryHistoryComponent;
