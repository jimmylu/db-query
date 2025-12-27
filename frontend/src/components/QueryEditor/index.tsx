import React, { useRef, useState, useEffect } from 'react';
import Editor from '@monaco-editor/react';
import { Card, Button, Space, Tooltip, Slider, message } from 'antd';
import { CodeOutlined, ExpandOutlined, CompressOutlined, FontSizeOutlined, FormatPainterOutlined, BulbOutlined, BulbFilled } from '@ant-design/icons';
import { formatAndUppercaseSQL } from '../../utils/sqlFormatter';

interface QueryEditorProps {
  value: string;
  onChange: (value: string) => void;
  height?: string;
}

export const QueryEditor: React.FC<QueryEditorProps> = ({
  value,
  onChange,
  height: initialHeight = '300px',
}) => {
  const editorRef = useRef<any>(null);
  const [height, setHeight] = useState<string>(initialHeight);
  const [fontSize, setFontSize] = useState<number>(14);
  const [isExpanded, setIsExpanded] = useState<boolean>(false);
  const [isDarkMode, setIsDarkMode] = useState<boolean>(() => {
    // Load from localStorage or default to true (dark mode)
    const saved = localStorage.getItem('editor-theme');
    return saved ? saved === 'dark' : true;
  });

  // Available height presets
  const heightPresets = {
    small: '200px',
    medium: '300px',
    large: '500px',
    xlarge: '700px',
  };

  const handleEditorDidMount = (editor: any) => {
    editorRef.current = editor;

    // Configure editor options
    editor.updateOptions({
      minimap: { enabled: false },
      fontSize,
      lineNumbers: 'on',
      roundedSelection: false,
      scrollBeyondLastLine: false,
      automaticLayout: true,
    });

    // Add SQL keywords suggestions
    const sqlKeywords = [
      'SELECT', 'FROM', 'WHERE', 'JOIN', 'LEFT JOIN', 'RIGHT JOIN', 'INNER JOIN',
      'GROUP BY', 'ORDER BY', 'HAVING', 'LIMIT', 'OFFSET', 'AS', 'ON',
      'AND', 'OR', 'NOT', 'IN', 'BETWEEN', 'LIKE', 'IS NULL', 'IS NOT NULL',
      'COUNT', 'SUM', 'AVG', 'MAX', 'MIN', 'DISTINCT', 'CASE', 'WHEN', 'THEN', 'ELSE', 'END',
    ];

    // Add custom SQL completions
    const monaco = (window as any).monaco;
    if (monaco) {
      monaco.languages.registerCompletionItemProvider('sql', {
        provideCompletionItems: () => {
          return {
            suggestions: sqlKeywords.map(keyword => ({
              label: keyword,
              kind: monaco.languages.CompletionItemKind.Keyword,
              insertText: keyword,
              detail: 'SQL Keyword',
            })),
          };
        },
      });
    }
  };

  // Update font size in editor when changed
  useEffect(() => {
    if (editorRef.current) {
      editorRef.current.updateOptions({ fontSize });
    }
  }, [fontSize]);

  const toggleExpand = () => {
    if (isExpanded) {
      setHeight(initialHeight);
    } else {
      setHeight(heightPresets.xlarge);
    }
    setIsExpanded(!isExpanded);
  };

  const handleHeightChange = (size: 'small' | 'medium' | 'large') => {
    setHeight(heightPresets[size]);
    setIsExpanded(false);
  };

  const handleFormat = () => {
    if (!value || !value.trim()) {
      message.warning('请先输入SQL查询');
      return;
    }

    try {
      const formatted = formatAndUppercaseSQL(value);
      onChange(formatted);
      message.success('SQL格式化成功');
    } catch (error: any) {
      message.error(`格式化失败: ${error.message}`);
    }
  };

  const toggleTheme = () => {
    const newTheme = !isDarkMode;
    setIsDarkMode(newTheme);
    localStorage.setItem('editor-theme', newTheme ? 'dark' : 'light');
  };

  return (
    <Card
      title={
        <Space>
          <CodeOutlined />
          <span>SQL 查询编辑器</span>
        </Space>
      }
      extra={
        <Space size="small">
          {/* Format SQL */}
          <Tooltip title="格式化SQL">
            <Button
              size="small"
              icon={<FormatPainterOutlined />}
              onClick={handleFormat}
            >
              格式化
            </Button>
          </Tooltip>

          {/* Font size control */}
          <Tooltip title="字体大小">
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, minWidth: 120 }}>
              <FontSizeOutlined />
              <Slider
                min={10}
                max={20}
                step={1}
                value={fontSize}
                onChange={setFontSize}
                style={{ width: 80, margin: 0 }}
              />
              <span style={{ fontSize: 12, color: '#999', minWidth: 30 }}>{fontSize}px</span>
            </div>
          </Tooltip>

          {/* Height presets */}
          <Tooltip title="编辑器高度">
            <Space.Compact>
              <Button
                size="small"
                type={height === heightPresets.small ? 'primary' : 'default'}
                onClick={() => handleHeightChange('small')}
              >
                小
              </Button>
              <Button
                size="small"
                type={height === heightPresets.medium ? 'primary' : 'default'}
                onClick={() => handleHeightChange('medium')}
              >
                中
              </Button>
              <Button
                size="small"
                type={height === heightPresets.large ? 'primary' : 'default'}
                onClick={() => handleHeightChange('large')}
              >
                大
              </Button>
            </Space.Compact>
          </Tooltip>

          {/* Expand/Collapse toggle */}
          <Tooltip title={isExpanded ? '收起' : '展开'}>
            <Button
              size="small"
              icon={isExpanded ? <CompressOutlined /> : <ExpandOutlined />}
              onClick={toggleExpand}
            />
          </Tooltip>

          {/* Theme toggle */}
          <Tooltip title={isDarkMode ? '切换到亮色模式' : '切换到暗色模式'}>
            <Button
              size="small"
              icon={isDarkMode ? <BulbFilled /> : <BulbOutlined />}
              onClick={toggleTheme}
            />
          </Tooltip>
        </Space>
      }
      style={{ marginBottom: 16 }}
    >
      <Editor
        height={height}
        defaultLanguage="sql"
        value={value}
        onChange={(val) => onChange(val || '')}
        onMount={handleEditorDidMount}
        theme={isDarkMode ? 'vs-dark' : 'vs-light'}
        options={{
          minimap: { enabled: false },
          fontSize,
          lineNumbers: 'on',
          roundedSelection: false,
          scrollBeyondLastLine: false,
          automaticLayout: true,
          wordWrap: 'on',
          formatOnPaste: true,
          formatOnType: true,
          suggestOnTriggerCharacters: true,
          quickSuggestions: true,
          tabCompletion: 'on',
          acceptSuggestionOnEnter: 'on',
        }}
      />
    </Card>
  );
};

