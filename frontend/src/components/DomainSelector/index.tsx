import React, { useState, useEffect } from 'react';
import { Select, message, Space, Tag } from 'antd';
import { FolderOutlined, DatabaseOutlined } from '@ant-design/icons';
import { domainService } from '../../services/domain';
import { DomainResponse } from '../../types';

interface DomainSelectorProps {
  value?: string;
  onChange?: (domainId: string, domain?: DomainResponse) => void;
  style?: React.CSSProperties;
  placeholder?: string;
}

export const DomainSelector: React.FC<DomainSelectorProps> = ({
  value,
  onChange,
  style,
  placeholder = '选择Domain',
}) => {
  const [domains, setDomains] = useState<DomainResponse[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadDomains();
  }, []);

  const loadDomains = async () => {
    setLoading(true);
    try {
      const data = await domainService.listDomains();
      setDomains(data);

      // Auto-select default domain if no value is set
      if (!value && data.length > 0) {
        const defaultDomain = data.find(d => d.id === 'default-domain-id') || data[0];
        onChange?.(defaultDomain.id, defaultDomain);
      }
    } catch (error: any) {
      message.error(`加载Domain列表失败: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (domainId: string) => {
    const selectedDomain = domains.find(d => d.id === domainId);
    onChange?.(domainId, selectedDomain);
  };

  return (
    <Select
      value={value}
      onChange={handleChange}
      loading={loading}
      placeholder={placeholder}
      style={{ minWidth: 200, ...style }}
      suffixIcon={<FolderOutlined />}
      optionLabelProp="label"
      allowClear
    >
      {domains.map((domain) => (
        <Select.Option
          key={domain.id}
          value={domain.id}
          label={
            <Space>
              <FolderOutlined />
              {domain.name}
            </Space>
          }
        >
          <Space direction="vertical" size={0} style={{ width: '100%' }}>
            <Space>
              <FolderOutlined />
              <strong>{domain.name}</strong>
              {domain.id === 'default-domain-id' && (
                <Tag color="default">默认</Tag>
              )}
            </Space>
            {domain.description && (
              <div style={{ fontSize: 12, color: '#999', paddingLeft: 20 }}>
                {domain.description}
              </div>
            )}
            <div style={{ fontSize: 12, color: '#666', paddingLeft: 20 }}>
              <DatabaseOutlined /> {domain.connection_count} 个连接
            </div>
          </Space>
        </Select.Option>
      ))}
    </Select>
  );
};
