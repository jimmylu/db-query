import React from 'react';
import { Card, CardProps } from 'antd';

/**
 * AWS-style container component
 * A white card with AWS-specific styling (sharp corners, subtle shadow)
 */
interface AWSContainerProps extends CardProps {
  children: React.ReactNode;
  noPadding?: boolean;
}

export const AWSContainer: React.FC<AWSContainerProps> = ({
  children,
  noPadding = false,
  style,
  ...props
}) => {
  return (
    <Card
      style={{
        borderRadius: 'var(--aws-border-radius)',
        boxShadow: 'var(--aws-shadow-default)',
        border: '1px solid var(--aws-border-gray)',
        background: '#FFFFFF',
        ...(noPadding && { padding: 0 }),
        ...style,
      }}
      bodyStyle={{
        ...(noPadding && { padding: 0 }),
      }}
      {...props}
    >
      {children}
    </Card>
  );
};

/**
 * AWS-style page header component
 * Similar to AWS Console page headers with title and actions
 */
interface AWSPageHeaderProps {
  title: string;
  description?: string;
  actions?: React.ReactNode;
}

export const AWSPageHeader: React.FC<AWSPageHeaderProps> = ({
  title,
  description,
  actions,
}) => {
  return (
    <div
      style={{
        padding: '20px 24px',
        background: '#FFFFFF',
        borderBottom: '1px solid var(--aws-border-gray)',
        marginBottom: '20px',
      }}
    >
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'flex-start',
        }}
      >
        <div>
          <h1
            style={{
              fontSize: '28px',
              fontWeight: 400,
              color: 'var(--aws-dark-navy)',
              margin: 0,
              marginBottom: description ? '8px' : 0,
            }}
          >
            {title}
          </h1>
          {description && (
            <p
              style={{
                fontSize: '14px',
                color: 'var(--aws-text-secondary)',
                margin: 0,
              }}
            >
              {description}
            </p>
          )}
        </div>
        {actions && <div>{actions}</div>}
      </div>
    </div>
  );
};

/**
 * AWS-style section header
 * Used within containers to separate content sections
 */
interface AWSSectionHeaderProps {
  title: string;
  actions?: React.ReactNode;
  divider?: boolean;
}

export const AWSSectionHeader: React.FC<AWSSectionHeaderProps> = ({
  title,
  actions,
  divider = true,
}) => {
  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '12px 0',
        borderBottom: divider ? '1px solid var(--aws-border-gray)' : 'none',
        marginBottom: '16px',
      }}
    >
      <h3
        style={{
          fontSize: '18px',
          fontWeight: 600,
          color: 'var(--aws-dark-navy)',
          margin: 0,
        }}
      >
        {title}
      </h3>
      {actions && <div>{actions}</div>}
    </div>
  );
};
