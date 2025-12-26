import React, { useState, useEffect } from 'react';
import { Button, Space, message, Select, Typography, Row, Col, Tabs } from 'antd';
import { PlayCircleOutlined, ClearOutlined } from '@ant-design/icons';
import { QueryEditor } from '../components/QueryEditor';
import { QueryResults } from '../components/QueryResults';
import { NaturalLanguageQuery } from '../components/NaturalLanguageQuery';
import { queryService } from '../services/query';
import { QueryResult } from '../types';
import { connectionService } from '../services/connection';
import { DatabaseConnection } from '../types';

const { Title } = Typography;

export const QueryPage: React.FC = () => {
  const [query, setQuery] = useState('');
  const [queryResult, setQueryResult] = useState<QueryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [nlLoading, setNlLoading] = useState(false);
  const [generatedSql, setGeneratedSql] = useState<string | undefined>();
  const [nlError, setNlError] = useState<string | undefined>();
  const [connections, setConnections] = useState<DatabaseConnection[]>([]);
  const [selectedConnectionId, setSelectedConnectionId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'sql' | 'nl'>('sql');

  // Load connections on mount
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

  const handleExecute = async () => {
    if (!selectedConnectionId) {
      message.warning('请先选择一个数据库连接');
      return;
    }

    if (!query.trim()) {
      message.warning('请输入 SQL 查询');
      return;
    }

    setLoading(true);
    setQueryResult(null);

    try {
      const result = await queryService.executeQuery(selectedConnectionId, query);
      setQueryResult(result.query);
      
      if (result.query.status === 'completed') {
        message.success(`查询成功！返回 ${result.query.row_count || 0} 行数据`);
      } else if (result.query.status === 'failed') {
        message.error(`查询失败: ${result.query.error_message || '未知错误'}`);
      }
    } catch (error: any) {
      const errorMessage = error.response?.data?.error?.message || error.message || '查询执行失败';
      message.error(`查询失败: ${errorMessage}`);
      
      // Set error result
      setQueryResult({
        id: '',
        connection_id: selectedConnectionId,
        query_text: query,
        is_llm_generated: false,
        status: 'failed',
        error_message: errorMessage,
        limit_applied: false,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleClear = () => {
    setQuery('');
    setQueryResult(null);
    setGeneratedSql(undefined);
    setNlError(undefined);
  };

  const handleNaturalLanguageQuery = async (question: string) => {
    if (!selectedConnectionId) {
      message.warning('请先选择一个数据库连接');
      return;
    }

    setNlLoading(true);
    setQueryResult(null);
    setGeneratedSql(undefined);
    setNlError(undefined);

    try {
      const result = await queryService.executeNaturalLanguageQuery(selectedConnectionId, question);
      
      // Set generated SQL
      if (result.generated_sql) {
        setGeneratedSql(result.generated_sql);
        // Also set it in the SQL editor
        setQuery(result.generated_sql);
        setActiveTab('sql');
      }
      
      setQueryResult(result.query);
      
      if (result.query.status === 'completed') {
        message.success(`查询成功！返回 ${result.query.row_count || 0} 行数据`);
      } else if (result.query.status === 'failed') {
        message.error(`查询失败: ${result.query.error_message || '未知错误'}`);
        setNlError(result.query.error_message || '查询执行失败');
      }
    } catch (error: any) {
      const errorMessage = error.response?.data?.error?.message || error.message || '查询生成失败';
      message.error(`查询失败: ${errorMessage}`);
      setNlError(errorMessage);
      
      // Set error result
      setQueryResult({
        id: '',
        connection_id: selectedConnectionId,
        query_text: '',
        is_llm_generated: true,
        status: 'failed',
        error_message: errorMessage,
        limit_applied: false,
      });
    } finally {
      setNlLoading(false);
    }
  };

  return (
    <div style={{ padding: '24px' }}>
      <Row gutter={[24, 24]}>
        <Col xs={24}>
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <div>
              <Title level={4}>数据库查询</Title>
              <Space style={{ marginBottom: 16 }}>
                <Select
                  placeholder="选择数据库连接"
                  style={{ width: 300 }}
                  value={selectedConnectionId}
                  onChange={setSelectedConnectionId}
                  options={connections.map((conn) => ({
                    label: conn.name || conn.connection_url,
                    value: conn.id,
                  }))}
                />
                <Button
                  icon={<ClearOutlined />}
                  onClick={handleClear}
                >
                  清空
                </Button>
              </Space>
            </div>

            <Tabs
              activeKey={activeTab}
              onChange={(key) => setActiveTab(key as 'sql' | 'nl')}
              items={[
                {
                  key: 'sql',
                  label: 'SQL 查询',
                  children: (
                    <Space direction="vertical" style={{ width: '100%' }} size="middle">
                      <QueryEditor value={query} onChange={setQuery} height="300px" />
                      <Button
                        type="primary"
                        icon={<PlayCircleOutlined />}
                        onClick={handleExecute}
                        loading={loading}
                        disabled={!selectedConnectionId || !query.trim()}
                        block
                      >
                        执行查询
                      </Button>
                    </Space>
                  ),
                },
                {
                  key: 'nl',
                  label: '自然语言查询',
                  children: (
                    <NaturalLanguageQuery
                      onExecute={handleNaturalLanguageQuery}
                      loading={nlLoading}
                      generatedSql={generatedSql}
                      error={nlError}
                    />
                  ),
                },
              ]}
            />

            <QueryResults queryResult={queryResult} loading={loading || nlLoading} />
          </Space>
        </Col>
      </Row>
    </div>
  );
};

