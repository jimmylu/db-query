import React, { useRef } from 'react';
import Editor from '@monaco-editor/react';
import { Card } from 'antd';
import { CodeOutlined } from '@ant-design/icons';

interface QueryEditorProps {
  value: string;
  onChange: (value: string) => void;
  height?: string;
}

export const QueryEditor: React.FC<QueryEditorProps> = ({
  value,
  onChange,
  height = '400px',
}) => {
  const editorRef = useRef<any>(null);

  const handleEditorDidMount = (editor: any) => {
    editorRef.current = editor;
    // Configure editor options
    editor.updateOptions({
      minimap: { enabled: false },
      fontSize: 14,
      lineNumbers: 'on',
      roundedSelection: false,
      scrollBeyondLastLine: false,
      automaticLayout: true,
    });
  };

  return (
    <Card
      title={
        <span>
          <CodeOutlined /> SQL 查询编辑器
        </span>
      }
      style={{ marginBottom: 16 }}
    >
      <Editor
        height={height}
        defaultLanguage="sql"
        value={value}
        onChange={(val) => onChange(val || '')}
        onMount={handleEditorDidMount}
        theme="vs-dark"
        options={{
          minimap: { enabled: false },
          fontSize: 14,
          lineNumbers: 'on',
          roundedSelection: false,
          scrollBeyondLastLine: false,
          automaticLayout: true,
          wordWrap: 'on',
          formatOnPaste: true,
          formatOnType: true,
        }}
      />
    </Card>
  );
};

