import React from 'react';
import { Breadcrumb } from 'antd';
import { Link, useLocation } from 'react-router-dom';
import { HomeOutlined } from '@ant-design/icons';

/**
 * AWS-style breadcrumb navigation
 * Automatically generates breadcrumbs based on current route
 */
export const AWSDynamicBreadcrumbs: React.FC = () => {
  const location = useLocation();
  const pathSegments = location.pathname.split('/').filter(Boolean);

  // Mapping of route segments to display names
  const routeNameMap: Record<string, string> = {
    '': '首页',
    'data-explore': '数据探索',
    'connections': '数据集列表',
  };

  const breadcrumbItems = [
    {
      title: (
        <Link to="/" style={{ color: 'var(--aws-pacific-blue)' }}>
          <HomeOutlined style={{ marginRight: 4 }} />
          首页
        </Link>
      ),
    },
  ];

  // Build breadcrumb items from path segments
  let currentPath = '';
  pathSegments.forEach((segment, index) => {
    currentPath += `/${segment}`;
    const isLast = index === pathSegments.length - 1;
    const displayName = routeNameMap[segment] || segment;

    breadcrumbItems.push({
      title: isLast ? (
        <span style={{ color: 'var(--aws-dark-navy)', fontWeight: 600 }}>
          {displayName}
        </span>
      ) : (
        <Link to={currentPath} style={{ color: 'var(--aws-pacific-blue)' }}>
          {displayName}
        </Link>
      ),
    });
  });

  // Only show breadcrumbs if not on home page
  if (pathSegments.length === 0) {
    return null;
  }

  return (
    <div
      style={{
        padding: '12px 24px',
        background: '#FFFFFF',
        borderBottom: '1px solid var(--aws-border-gray)',
      }}
    >
      <Breadcrumb items={breadcrumbItems} separator="/" />
    </div>
  );
};
