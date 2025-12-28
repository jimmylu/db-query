import React, { useState, useEffect } from 'react';
import { Layout, Space, Typography, Button, Modal } from 'antd';
import { SettingOutlined } from '@ant-design/icons';
import { DomainSelector } from '../DomainSelector';
import { domainService } from '../../services/domain';
import type { DomainResponse } from '../../types';
import { DomainsPage } from '../../pages/DomainsPage';

const { Header } = Layout;
const { Text } = Typography;

export const CustomHeader: React.FC = () => {
  // Initialize selectedDomainId from localStorage synchronously
  const initialDomainId = localStorage.getItem('selectedDomainId');
  const [selectedDomainId, setSelectedDomainId] = useState<string | null>(initialDomainId);
  const [domainsModalVisible, setDomainsModalVisible] = useState(false);

  useEffect(() => {
    // Load and auto-select default domain on mount
    const loadDefaultDomain = async () => {
      try {
        const domains = await domainService.listDomains();
        if (domains.length > 0) {
          const defaultDomain = domains.find(d => d.id === 'default-domain-id') || domains[0];
          setSelectedDomainId(defaultDomain.id);

          // Store in localStorage for persistence across pages
          localStorage.setItem('selectedDomainId', defaultDomain.id);

          // Dispatch event so other components are notified
          window.dispatchEvent(new CustomEvent('domainChanged', {
            detail: { domainId: defaultDomain.id, domain: defaultDomain }
          }));
        }
      } catch (error) {
        console.error('Failed to load default domain:', error);
      }
    };

    // Note: selectedDomainId is already initialized from localStorage in useState
    console.log('CustomHeader: Initialized with domain from localStorage:', initialDomainId);

    if (initialDomainId) {
      // Immediately dispatch event with stored domain ID
      // This ensures other components get notified right away
      window.dispatchEvent(new CustomEvent('domainChanged', {
        detail: { domainId: initialDomainId, domain: null }
      }));
      console.log('CustomHeader: Immediately dispatched domainChanged event with:', initialDomainId);

      // Verify the domain still exists (async)
      domainService.getDomain(initialDomainId)
        .then((domain) => {
          console.log('CustomHeader: Domain verified:', domain);
          // Dispatch again with full domain data
          window.dispatchEvent(new CustomEvent('domainChanged', {
            detail: { domainId: initialDomainId, domain }
          }));
        })
        .catch(() => {
          // If stored domain is invalid, load default
          console.log('CustomHeader: Stored domain invalid, loading default');
          loadDefaultDomain();
        });
    } else {
      console.log('CustomHeader: No stored domain, loading default');
      loadDefaultDomain();
    }
  }, []);

  const handleDomainChange = (domainId: string, domain?: DomainResponse) => {
    setSelectedDomainId(domainId);

    // Persist selection to localStorage
    if (domainId) {
      localStorage.setItem('selectedDomainId', domainId);
    } else {
      localStorage.removeItem('selectedDomainId');
    }

    // Dispatch custom event to notify other components
    window.dispatchEvent(new CustomEvent('domainChanged', {
      detail: { domainId, domain }
    }));
  };

  return (
    <Header
      style={{
        display: 'flex',
        justifyContent: 'flex-end',
        alignItems: 'center',
        padding: '0 24px',
        background: 'var(--aws-squid-ink)',
        borderBottom: 'none',
        height: '48px',
        boxShadow: 'var(--aws-shadow-sticky)',
      }}
    >
      <Space size="middle">
        <Text style={{ fontSize: '14px', color: '#FFFFFF', fontWeight: 500 }}>
          当前工作域:
        </Text>
        <DomainSelector
          value={selectedDomainId || undefined}
          onChange={handleDomainChange}
          placeholder="选择Domain"
          style={{ width: 280 }}
        />
        <Button
          icon={<SettingOutlined />}
          onClick={() => setDomainsModalVisible(true)}
          type="default"
          style={{
            background: '#37475A',
            borderColor: '#687078',
            color: '#FFFFFF',
          }}
        >
          管理Domain
        </Button>
      </Space>

      <Modal
        title="Domain 管理"
        open={domainsModalVisible}
        onCancel={() => setDomainsModalVisible(false)}
        footer={null}
        width={1000}
        destroyOnClose
      >
        <DomainsPage />
      </Modal>
    </Header>
  );
};
