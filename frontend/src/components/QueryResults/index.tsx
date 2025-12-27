import React from 'react';
import { Card, Table, Tag, Typography, Empty, Alert, Button, Space, Dropdown, message } from 'antd';
import { TableOutlined, CloseCircleOutlined, ClockCircleOutlined, DownloadOutlined } from '@ant-design/icons';
import type { MenuProps } from 'antd';
import { QueryResult } from '../../types';
import { exportToCSV, exportToJSON, generateFilename } from '../../utils/exportData';

const { Text } = Typography;

interface QueryResultsProps {
  queryResult: QueryResult | null;
  loading?: boolean;
}

export const QueryResults: React.FC<QueryResultsProps> = ({
  queryResult,
  loading = false,
}) => {
  const handleExport = (format: 'csv' | 'json') => {
    if (!queryResult?.results || queryResult.results.length === 0) {
      message.warning('没有数据可供导出');
      return;
    }

    try {
      if (format === 'csv') {
        const filename = generateFilename('query_results', 'csv');
        exportToCSV(queryResult.results, filename);
        message.success(`已导出 ${queryResult.results.length} 行数据到 ${filename}`);
      } else {
        const filename = generateFilename('query_results', 'json');
        exportToJSON(queryResult.results, filename);
        message.success(`已导出 ${queryResult.results.length} 行数据到 ${filename}`);
      }
    } catch (error: any) {
      message.error(`导出失败: ${error.message}`);
    }
  };

  const exportMenuItems: MenuProps['items'] = [
    {
      key: 'csv',
      label: '导出为 CSV',
      onClick: () => handleExport('csv'),
    },
    {
      key: 'json',
      label: '导出为 JSON',
      onClick: () => handleExport('json'),
    },
  ];

  if (!queryResult) {
    return (
      <Card
        title={
          <span>
            <TableOutlined /> 查询结果
          </span>
        }
      >
        <Empty description="执行查询以查看结果" />
      </Card>
    );
  }

  // Show error state
  if (queryResult.status === 'failed') {
    return (
      <Card
        title={
          <span>
            <TableOutlined /> 查询结果
          </span>
        }
      >
        <Alert
          message="查询执行失败"
          description={queryResult.error_message || '未知错误'}
          type="error"
          icon={<CloseCircleOutlined />}
          showIcon
        />
      </Card>
    );
  }

  // Show loading state
  if (loading || queryResult.status === 'executing' || queryResult.status === 'pending') {
    return (
      <Card
        title={
          <span>
            <TableOutlined /> 查询结果
          </span>
        }
      >
        <div style={{ textAlign: 'center', padding: '40px' }}>
          <ClockCircleOutlined style={{ fontSize: 48, color: '#1890ff' }} />
          <div style={{ marginTop: 16 }}>
            <Text>正在执行查询...</Text>
          </div>
        </div>
      </Card>
    );
  }

  // Show results
  if (queryResult.status === 'completed' && queryResult.results && queryResult.results.length > 0) {
    // Extract columns from first row
    const columns = Object.keys(queryResult.results[0]).map((key) => ({
      title: key,
      dataIndex: key,
      key,
      ellipsis: true,
      render: (text: any) => {
        if (text === null || text === undefined) {
          return <Text type="secondary" italic>NULL</Text>;
        }
        if (typeof text === 'object') {
          return JSON.stringify(text);
        }
        return String(text);
      },
    }));

    // Prepare data source
    const dataSource = queryResult.results.map((row, index) => ({
      key: index,
      ...row,
    }));

    return (
      <Card
        title={
          <span>
            <TableOutlined /> 查询结果
            {queryResult.limit_applied && (
              <Tag color="orange" style={{ marginLeft: 8 }}>
                已自动添加 LIMIT 1000
              </Tag>
            )}
            {queryResult.is_llm_generated && (
              <Tag color="blue" style={{ marginLeft: 8 }}>
                LLM 生成
              </Tag>
            )}
          </span>
        }
        extra={
          <Space>
            <Text type="secondary">
              共 {queryResult.row_count || 0} 行
              {queryResult.execution_time_ms !== undefined && (
                <> · 执行时间: {queryResult.execution_time_ms}ms</>
              )}
            </Text>
            <Dropdown menu={{ items: exportMenuItems }} placement="bottomRight">
              <Button icon={<DownloadOutlined />} size="small">
                导出数据
              </Button>
            </Dropdown>
          </Space>
        }
      >
        <Table
          columns={columns}
          dataSource={dataSource}
          pagination={{
            pageSize: 100,
            pageSizeOptions: ['50', '100', '200', '500', '1000'],
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条记录`,
          }}
          scroll={{ x: 'max-content', y: 600 }}
          size="small"
          virtual
          sticky
        />
      </Card>
    );
  }

  // Empty results
  return (
    <Card
      title={
        <span>
          <TableOutlined /> 查询结果
        </span>
      }
    >
      <Empty description="查询成功，但没有返回数据" />
    </Card>
  );
};

