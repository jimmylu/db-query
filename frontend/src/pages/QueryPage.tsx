import React, { useState, useEffect } from 'react';
import { Button, Space, message, Select, Typography, Row, Col, Tabs, Alert, Tag, Collapse, Drawer, List, Tooltip, Empty } from 'antd';
import { PlayCircleOutlined, ClearOutlined, ThunderboltOutlined, InfoCircleOutlined, HistoryOutlined, ClockCircleOutlined, CheckCircleOutlined, CloseCircleOutlined, DeleteOutlined } from '@ant-design/icons';
import { QueryEditor } from '../components/QueryEditor';
import { QueryResults } from '../components/QueryResults';
import { NaturalLanguageQuery } from '../components/NaturalLanguageQuery';
import { queryService } from '../services/query';
import { unifiedQueryService, DatabaseType } from '../services/unified_query';
import type { UnifiedQueryResponse } from '../services/unified_query';
import { QueryResult } from '../types';
import { connectionService } from '../services/connection';
import { DatabaseConnection } from '../types';
import { QueryHistoryManager, QueryHistoryItem } from '../utils/queryHistory';

const { Title, Text, Paragraph } = Typography;
const { Panel } = Collapse;

export const QueryPage: React.FC = () => {
  const [query, setQuery] = useState('');
  const [queryResult, setQueryResult] = useState<QueryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [nlLoading, setNlLoading] = useState(false);
  const [generatedSql, setGeneratedSql] = useState<string | undefined>();
  const [nlError, setNlError] = useState<string | undefined>();
  const [connections, setConnections] = useState<DatabaseConnection[]>([]);
  const [selectedConnectionId, setSelectedConnectionId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'sql' | 'nl' | 'unified'>('sql');

  // Unified SQL states
  const [useUnifiedQuery, setUseUnifiedQuery] = useState(false);
  const [unifiedResponse, setUnifiedResponse] = useState<UnifiedQueryResponse | null>(null);
  const [databaseType, setDatabaseType] = useState<DatabaseType | null>(null);

  // Query history states
  const [showHistory, setShowHistory] = useState(false);
  const [queryHistory, setQueryHistory] = useState<QueryHistoryItem[]>([]);

  // Get selected connection
  const selectedConnection = connections.find(c => c.id === selectedConnectionId);

  // Auto-detect database type when connection changes
  useEffect(() => {
    if (selectedConnection) {
      try {
        const dbType = unifiedQueryService.getDatabaseType(selectedConnection.database_type);
        setDatabaseType(dbType);

        // Auto-enable unified query for supported databases
        const isSupported = unifiedQueryService.isUnifiedQuerySupported(dbType);
        if (isSupported && !useUnifiedQuery) {
          setUseUnifiedQuery(true);
        }
      } catch (error) {
        console.error('Failed to detect database type:', error);
        setDatabaseType(null);
        setUseUnifiedQuery(false);
      }
    }
  }, [selectedConnection]);

  // Load connections on mount
  useEffect(() => {
    loadConnections();
    loadQueryHistory();
  }, []);

  // Reload history when showing the drawer
  useEffect(() => {
    if (showHistory) {
      loadQueryHistory();
    }
  }, [showHistory]);

  // Keyboard shortcut: Cmd/Ctrl + Enter to execute query
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
        e.preventDefault();
        if (activeTab === 'sql' && selectedConnectionId && query.trim() && !loading) {
          handleExecute();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [activeTab, selectedConnectionId, query, loading]);

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

  const loadQueryHistory = () => {
    const history = QueryHistoryManager.getRecentQueries(20);
    setQueryHistory(history);
  };

  const handleLoadHistoryQuery = (historyItem: QueryHistoryItem) => {
    setQuery(historyItem.query);
    setSelectedConnectionId(historyItem.connectionId);
    setShowHistory(false);
    setActiveTab('sql');
    message.success('已加载历史查询');
  };

  const handleDeleteHistoryItem = (id: string) => {
    QueryHistoryManager.deleteQuery(id);
    loadQueryHistory();
    message.success('已删除历史记录');
  };

  const handleClearHistory = () => {
    QueryHistoryManager.clearHistory();
    loadQueryHistory();
    message.success('已清空历史记录');
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
    setUnifiedResponse(null);

    try {
      // Use unified query if enabled and database type is detected
      if (useUnifiedQuery && databaseType) {
        const response = await unifiedQueryService.executeUnifiedQuery(selectedConnectionId, {
          query,
          database_type: databaseType,
          timeout_secs: 30,
          apply_limit: true,
          limit_value: 1000,
        });

        setUnifiedResponse(response);

        // Convert to QueryResult format for display
        const queryResult: QueryResult = {
          id: '',
          connection_id: selectedConnectionId,
          query_text: response.translated_query,
          is_llm_generated: false,
          status: 'completed',
          results: response.results,
          row_count: response.row_count,
          execution_time_ms: response.execution_time_ms,
          limit_applied: response.limit_applied,
        };
        setQueryResult(queryResult);

        // Save to history
        QueryHistoryManager.addQuery({
          query,
          connectionId: selectedConnectionId,
          connectionName: selectedConnection?.name || selectedConnection?.connection_url || '',
          success: true,
          rowCount: response.row_count,
          executionTimeMs: response.execution_time_ms,
        });

        message.success(
          <span>
            查询成功！返回 {response.row_count} 行数据
            {response.limit_applied && ' (已自动添加 LIMIT)'}
            <br />
            <Text type="secondary" style={{ fontSize: '12px' }}>
              执行时间: {response.execution_time_ms}ms
            </Text>
          </span>
        );
      } else {
        // Use legacy query execution
        const result = await queryService.executeQuery(selectedConnectionId, query);
        setQueryResult(result.query);

        if (result.query.status === 'completed') {
          // Save to history
          QueryHistoryManager.addQuery({
            query,
            connectionId: selectedConnectionId,
            connectionName: selectedConnection?.name || selectedConnection?.connection_url || '',
            success: true,
            rowCount: result.query.row_count,
            executionTimeMs: result.query.execution_time_ms,
          });

          message.success(`查询成功！返回 ${result.query.row_count || 0} 行数据`);
        } else if (result.query.status === 'failed') {
          // Save failed query to history
          QueryHistoryManager.addQuery({
            query,
            connectionId: selectedConnectionId,
            connectionName: selectedConnection?.name || selectedConnection?.connection_url || '',
            success: false,
          });

          message.error(`查询失败: ${result.query.error_message || '未知错误'}`);
        }
      }
    } catch (error: any) {
      const errorMessage = error.response?.data?.error?.message || error.message || '查询执行失败';
      message.error(
        <span>
          查询失败: {errorMessage}
          {useUnifiedQuery && (
            <Text type="secondary" style={{ fontSize: '12px', display: 'block' }}>
              提示: 使用统一SQL语法时，请确保SQL符合DataFusion标准
            </Text>
          )}
        </span>
      );

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
    setUnifiedResponse(null);
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
              <Space style={{ marginBottom: 16 }} wrap>
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

                {/* Database Type Indicator */}
                {selectedConnection && databaseType && (
                  <Tag
                    icon={<InfoCircleOutlined />}
                    color={unifiedQueryService.isUnifiedQuerySupported(databaseType) ? 'green' : 'default'}
                  >
                    {unifiedQueryService.getDatabaseTypeName(databaseType)}
                    {unifiedQueryService.isUnifiedQuerySupported(databaseType) && ' (支持统一SQL)'}
                  </Tag>
                )}

                {/* Unified SQL Toggle */}
                {databaseType && unifiedQueryService.isUnifiedQuerySupported(databaseType) && (
                  <Button
                    type={useUnifiedQuery ? 'primary' : 'default'}
                    icon={<ThunderboltOutlined />}
                    onClick={() => setUseUnifiedQuery(!useUnifiedQuery)}
                  >
                    统一SQL语法
                  </Button>
                )}

                <Button
                  icon={<HistoryOutlined />}
                  onClick={() => setShowHistory(true)}
                >
                  查询历史
                </Button>

                <Button
                  icon={<ClearOutlined />}
                  onClick={handleClear}
                >
                  清空
                </Button>
              </Space>

              {/* Unified SQL Info Alert */}
              {useUnifiedQuery && databaseType && (
                <Alert
                  message="使用统一SQL语法"
                  description={
                    <span>
                      您的查询将使用DataFusion标准SQL语法，并自动翻译为{' '}
                      {unifiedQueryService.getDatabaseTypeName(databaseType)} 方言执行。
                      <br />
                      <Text type="secondary" style={{ fontSize: '12px' }}>
                        支持跨数据库通用的SQL语法，如 INTERVAL, CURRENT_DATE 等标准函数
                      </Text>
                    </span>
                  }
                  type="info"
                  showIcon
                  closable
                  style={{ marginTop: 8 }}
                />
              )}
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
                        执行查询 (⌘/Ctrl + Enter)
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

            {/* Dialect Translation Info */}
            {unifiedResponse && (
              <Collapse
                items={[
                  {
                    key: 'translation',
                    label: (
                      <Space>
                        <ThunderboltOutlined style={{ color: '#1890ff' }} />
                        <Text strong>查看SQL方言翻译</Text>
                        <Tag color="blue">
                          {unifiedQueryService.getDatabaseTypeName(unifiedResponse.database_type)}
                        </Tag>
                      </Space>
                    ),
                    children: (
                      <Space direction="vertical" style={{ width: '100%' }}>
                        <div>
                          <Text type="secondary" style={{ fontSize: '12px' }}>
                            原始DataFusion SQL:
                          </Text>
                          <Paragraph
                            code
                            copyable
                            style={{
                              background: '#f5f5f5',
                              padding: '8px 12px',
                              marginTop: '4px',
                              marginBottom: '12px',
                              borderRadius: '4px',
                            }}
                          >
                            {unifiedResponse.original_query}
                          </Paragraph>
                        </div>
                        <div>
                          <Text type="secondary" style={{ fontSize: '12px' }}>
                            翻译后的{unifiedQueryService.getDatabaseTypeName(unifiedResponse.database_type)}方言:
                          </Text>
                          <Paragraph
                            code
                            copyable
                            style={{
                              background: '#e6f7ff',
                              padding: '8px 12px',
                              marginTop: '4px',
                              borderRadius: '4px',
                            }}
                          >
                            {unifiedResponse.translated_query}
                          </Paragraph>
                        </div>
                        <div>
                          <Text type="secondary" style={{ fontSize: '12px' }}>
                            执行时间: {unifiedResponse.execution_time_ms}ms
                            {unifiedResponse.limit_applied && ' | 已自动添加 LIMIT'}
                          </Text>
                        </div>
                      </Space>
                    ),
                  },
                ]}
                style={{ marginBottom: 16 }}
              />
            )}

            <QueryResults queryResult={queryResult} loading={loading || nlLoading} />
          </Space>
        </Col>
      </Row>

      {/* Query History Drawer */}
      <Drawer
        title={
          <Space>
            <HistoryOutlined />
            <span>查询历史</span>
            <Tag color="blue">{queryHistory.length} 条记录</Tag>
          </Space>
        }
        placement="right"
        width={600}
        onClose={() => setShowHistory(false)}
        open={showHistory}
        extra={
          <Button
            danger
            size="small"
            onClick={handleClearHistory}
            disabled={queryHistory.length === 0}
          >
            清空全部
          </Button>
        }
      >
        {queryHistory.length === 0 ? (
          <Empty description="暂无查询历史" />
        ) : (
          <List
            dataSource={queryHistory}
            renderItem={(item) => (
              <List.Item
                actions={[
                  <Tooltip title="加载此查询">
                    <Button
                      type="link"
                      size="small"
                      onClick={() => handleLoadHistoryQuery(item)}
                    >
                      加载
                    </Button>
                  </Tooltip>,
                  <Tooltip title="删除">
                    <Button
                      type="link"
                      danger
                      size="small"
                      icon={<DeleteOutlined />}
                      onClick={() => handleDeleteHistoryItem(item.id)}
                    />
                  </Tooltip>,
                ]}
              >
                <List.Item.Meta
                  avatar={
                    item.success ? (
                      <CheckCircleOutlined style={{ fontSize: 20, color: '#52c41a' }} />
                    ) : (
                      <CloseCircleOutlined style={{ fontSize: 20, color: '#ff4d4f' }} />
                    )
                  }
                  title={
                    <Space direction="vertical" style={{ width: '100%' }} size={0}>
                      <Text
                        ellipsis
                        style={{
                          fontFamily: 'monospace',
                          fontSize: '13px',
                          display: 'block',
                          maxWidth: '400px',
                        }}
                      >
                        {item.query}
                      </Text>
                      <Space size={8} wrap>
                        <Tag color="blue" style={{ fontSize: '11px' }}>
                          {item.connectionName}
                        </Tag>
                        {item.success && item.rowCount !== undefined && (
                          <Tag color="green" style={{ fontSize: '11px' }}>
                            {item.rowCount} 行
                          </Tag>
                        )}
                        {item.success && item.executionTimeMs !== undefined && (
                          <Tag color="cyan" style={{ fontSize: '11px' }}>
                            {item.executionTimeMs}ms
                          </Tag>
                        )}
                      </Space>
                    </Space>
                  }
                  description={
                    <Text type="secondary" style={{ fontSize: '12px' }}>
                      <ClockCircleOutlined /> {new Date(item.timestamp).toLocaleString('zh-CN')}
                    </Text>
                  }
                />
              </List.Item>
            )}
          />
        )}
      </Drawer>
    </div>
  );
};

