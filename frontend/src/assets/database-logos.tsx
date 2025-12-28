// Database logos as React components
import React from 'react';

export const PostgreSQLLogo: React.FC<{ size?: number }> = ({ size = 48 }) => (
  <svg width={size} height={size} viewBox="0 0 48 48" fill="none">
    <path d="M37.8 18.3c-2.1-1.2-4.5-1.8-7.2-1.8-1.8 0-3.6.3-5.1.9-1.5.6-2.7 1.5-3.6 2.7-.9 1.2-1.3 2.6-1.3 4.2 0 1.2.3 2.4.9 3.4.6 1 1.5 1.8 2.7 2.4 1.2.6 2.7.9 4.5.9 1.5 0 2.7-.3 3.9-.9 1.2-.6 2.1-1.5 2.7-2.4.6-1 .9-2.1.9-3.3v-3.3c.9-.6 1.5-1.2 1.8-1.8.3-.6.6-1.2.6-1.8 0-.9-.3-1.5-.9-2.1-.6-.6-1.2-.9-2.1-.9-.6 0-1.2.3-1.8.6zm-7.2 8.4c-.9 0-1.5-.3-2.1-.9-.6-.6-.9-1.2-.9-2.1s.3-1.5.9-2.1c.6-.6 1.2-.9 2.1-.9.9 0 1.5.3 2.1.9.6.6.9 1.2.9 2.1s-.3 1.5-.9 2.1c-.6.6-1.2.9-2.1.9z" fill="#336791"/>
    <ellipse cx="24" cy="24" rx="18" ry="18" stroke="#336791" strokeWidth="2" fill="none"/>
    <text x="24" y="32" fontSize="10" fontWeight="bold" textAnchor="middle" fill="#336791">PG</text>
  </svg>
);

export const MySQLLogo: React.FC<{ size?: number }> = ({ size = 48 }) => (
  <svg width={size} height={size} viewBox="0 0 48 48" fill="none">
    <circle cx="24" cy="24" r="18" fill="#00758F"/>
    <path d="M18 18h12v12H18V18z" fill="white" opacity="0.3"/>
    <text x="24" y="30" fontSize="9" fontWeight="bold" textAnchor="middle" fill="white">MySQL</text>
  </svg>
);

export const DorisLogo: React.FC<{ size?: number }> = ({ size = 48 }) => (
  <svg width={size} height={size} viewBox="0 0 48 48" fill="none">
    <circle cx="24" cy="24" r="18" fill="#FF6B35"/>
    <path d="M15 20l9-6 9 6v12l-9 6-9-6V20z" fill="white" opacity="0.9"/>
    <text x="24" y="28" fontSize="8" fontWeight="bold" textAnchor="middle" fill="#FF6B35">Doris</text>
  </svg>
);

export const DruidLogo: React.FC<{ size?: number }> = ({ size = 48 }) => (
  <svg width={size} height={size} viewBox="0 0 48 48" fill="none">
    <circle cx="24" cy="24" r="18" fill="#29B7CB"/>
    <rect x="16" y="16" width="16" height="16" rx="2" fill="white" opacity="0.9"/>
    <text x="24" y="28" fontSize="8" fontWeight="bold" textAnchor="middle" fill="#29B7CB">Druid</text>
  </svg>
);

interface DatabaseLogoProps {
  type: 'postgresql' | 'mysql' | 'doris' | 'druid';
  size?: number;
}

export const DatabaseLogo: React.FC<DatabaseLogoProps> = ({ type, size = 48 }) => {
  switch (type) {
    case 'postgresql':
      return <PostgreSQLLogo size={size} />;
    case 'mysql':
      return <MySQLLogo size={size} />;
    case 'doris':
      return <DorisLogo size={size} />;
    case 'druid':
      return <DruidLogo size={size} />;
    default:
      return null;
  }
};
