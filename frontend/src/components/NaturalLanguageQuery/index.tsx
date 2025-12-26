import React, { useState } from 'react';
import { Card, Input, Button, Space, Typography, Alert, Tag } from 'antd';
import { MessageOutlined, SendOutlined, CodeOutlined } from '@ant-design/icons';

const { TextArea } = Input;
const { Text } = Typography;

interface NaturalLanguageQueryProps {
  onExecute: (question: string) => void;
  loading?: boolean;
  generatedSql?: string;
  error?: string;
}

export const NaturalLanguageQuery: React.FC<NaturalLanguageQueryProps> = ({
  onExecute,
  loading = false,
  generatedSql,
  error,
}) => {
  const [question, setQuestion] = useState('');

  const handleSubmit = () => {
    if (question.trim()) {
      onExecute(question.trim());
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      handleSubmit();
    }
  };

  return (
    <Card
      title={
        <Space>
          <MessageOutlined />
          <span>自然语言查询</span>
        </Space>
      }
      style={{ marginBottom: 16 }}
    >
      <Space direction="vertical" style={{ width: '100%' }} size="middle">
        <div>
          <Text type="secondary" style={{ fontSize: '12px' }}>
            用自然语言描述您想要查询的内容，系统将自动生成 SQL 查询
          </Text>
        </div>

        <TextArea
          rows={3}
          placeholder="例如：查询所有公司的名称和地址"
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          onKeyDown={handleKeyPress}
          disabled={loading}
        />

        <Space>
          <Button
            type="primary"
            icon={<SendOutlined />}
            onClick={handleSubmit}
            loading={loading}
            disabled={!question.trim()}
          >
            生成并执行查询
          </Button>
          <Button onClick={() => setQuestion('')} disabled={loading}>
            清空
          </Button>
        </Space>

        {error && (
          <Alert
            message="查询生成失败"
            description={error}
            type="error"
            showIcon
            closable
          />
        )}

        {generatedSql && !error && (
          <Alert
            message={
              <Space>
                <CodeOutlined />
                <span>生成的 SQL 查询</span>
                <Tag color="blue">LLM 生成</Tag>
              </Space>
            }
            description={
              <div style={{ marginTop: 8 }}>
                <Text code style={{ fontSize: '12px', whiteSpace: 'pre-wrap' }}>
                  {generatedSql}
                </Text>
              </div>
            }
            type="info"
            showIcon
            style={{ marginTop: 8 }}
          />
        )}
      </Space>
    </Card>
  );
};

