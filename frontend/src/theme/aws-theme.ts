import type { ThemeConfig } from 'antd';

/**
 * AWS-inspired theme configuration for Ant Design
 * Based on AWS Console color palette and design guidelines
 */
export const awsTheme: ThemeConfig = {
  token: {
    // Primary colors - AWS Blue
    colorPrimary: '#0972D3', // AWS primary blue
    colorInfo: '#0972D3',
    colorSuccess: '#037F0C', // AWS green
    colorWarning: '#FF9900', // AWS orange
    colorError: '#D13212', // AWS red

    // Background colors - AWS Console style
    colorBgBase: '#FFFFFF',
    colorBgContainer: '#FFFFFF',
    colorBgElevated: '#FFFFFF',
    colorBgLayout: '#F2F3F3', // AWS light gray background

    // Border colors
    colorBorder: '#D5DBDB', // AWS border gray
    colorBorderSecondary: '#E9EBED',

    // Text colors
    colorText: '#16191F', // AWS dark text
    colorTextSecondary: '#545B64', // AWS secondary text
    colorTextTertiary: '#879596',
    colorTextQuaternary: '#AAB7B8',

    // Spacing and sizing
    borderRadius: 2, // AWS uses sharp corners (minimal radius)
    borderRadiusLG: 4,
    borderRadiusSM: 2,

    // Font
    fontSize: 14,
    fontSizeHeading1: 28,
    fontSizeHeading2: 24,
    fontSizeHeading3: 20,
    fontSizeHeading4: 16,
    fontSizeHeading5: 14,
    fontFamily: '"Amazon Ember", "Helvetica Neue", Roboto, Arial, sans-serif',

    // Line height
    lineHeight: 1.5,
    lineHeightHeading1: 1.2,
    lineHeightHeading2: 1.3,
    lineHeightHeading3: 1.4,

    // Control heights
    controlHeight: 32,
    controlHeightLG: 40,
    controlHeightSM: 24,

    // Shadows - AWS uses subtle shadows
    boxShadow: '0 1px 1px 0 rgba(0, 28, 36, 0.3), 1px 1px 1px 0 rgba(0, 28, 36, 0.15), -1px 1px 1px 0 rgba(0, 28, 36, 0.15)',
    boxShadowSecondary: '0 2px 4px 0 rgba(0, 28, 36, 0.5)',
  },

  components: {
    // Layout
    Layout: {
      headerBg: '#232F3E', // AWS dark navy header
      headerColor: '#FFFFFF',
      headerHeight: 48,
      headerPadding: '0 20px',
      siderBg: '#232F3E', // AWS dark navy sidebar
      bodyBg: '#F2F3F3',
      footerBg: '#FFFFFF',
      footerPadding: '20px 50px',
    },

    // Menu
    Menu: {
      darkItemBg: '#232F3E',
      darkItemColor: '#AAB7B8',
      darkItemHoverBg: '#37475A',
      darkItemHoverColor: '#FFFFFF',
      darkItemSelectedBg: '#0972D3',
      darkItemSelectedColor: '#FFFFFF',
      itemBorderRadius: 2,
      itemMarginInline: 4,
    },

    // Button - AWS style buttons
    Button: {
      primaryColor: '#FFFFFF',
      primaryShadow: 'none',
      defaultBorderColor: '#AAB7B8',
      defaultColor: '#16191F',
      defaultBg: '#FFFFFF',
      defaultShadow: 'none',
      controlHeight: 32,
      controlHeightLG: 40,
      controlHeightSM: 24,
      borderRadius: 2,
      borderRadiusLG: 2,
      borderRadiusSM: 2,
      fontWeight: 600,
      paddingInline: 16,
      paddingInlineLG: 20,
      paddingInlineSM: 12,
    },

    // Card - AWS container style
    Card: {
      headerBg: '#FAFAFA',
      headerFontSize: 18,
      headerFontSizeSM: 16,
      headerHeight: 48,
      headerHeightSM: 40,
      boxShadow: '0 1px 1px 0 rgba(0, 28, 36, 0.3)',
      borderRadiusLG: 2,
      paddingLG: 20,
    },

    // Table - AWS data table style
    Table: {
      headerBg: '#FAFAFA',
      headerColor: '#16191F',
      headerSortActiveBg: '#E9EBED',
      headerSortHoverBg: '#F2F3F3',
      rowHoverBg: '#F9F9F9',
      borderColor: '#D5DBDB',
      borderRadius: 0, // AWS tables have sharp corners
      borderRadiusLG: 0,
      cellPaddingBlock: 12,
      cellPaddingInline: 16,
      headerSplitColor: '#D5DBDB',
      fixedHeaderSortActiveBg: '#E9EBED',
    },

    // Input
    Input: {
      borderRadius: 2,
      controlHeight: 32,
      controlHeightLG: 40,
      controlHeightSM: 24,
      paddingBlock: 6,
      paddingInline: 12,
      activeBorderColor: '#0972D3',
      hoverBorderColor: '#687078',
    },

    // Select
    Select: {
      borderRadius: 2,
      controlHeight: 32,
      controlHeightLG: 40,
      controlHeightSM: 24,
    },

    // Tabs - AWS style tabs
    Tabs: {
      itemColor: '#545B64',
      itemHoverColor: '#16191F',
      itemSelectedColor: '#0972D3',
      itemActiveColor: '#0972D3',
      inkBarColor: '#0972D3',
      cardBg: '#FFFFFF',
      cardHeight: 40,
      horizontalItemPadding: '12px 0',
      horizontalMargin: '0 0 16px 0',
    },

    // Modal
    Modal: {
      headerBg: '#FAFAFA',
      contentBg: '#FFFFFF',
      titleFontSize: 18,
      borderRadiusLG: 2,
    },

    // Badge
    Badge: {
      dotSize: 8,
      statusSize: 8,
    },

    // Tag
    Tag: {
      defaultBg: '#E9EBED',
      defaultColor: '#16191F',
      borderRadiusSM: 2,
    },

    // Alert
    Alert: {
      borderRadiusLG: 2,
      withDescriptionPadding: '16px 20px',
      withDescriptionIconSize: 24,
    },

    // Breadcrumb - AWS breadcrumb style
    Breadcrumb: {
      itemColor: '#545B64',
      lastItemColor: '#16191F',
      linkColor: '#0972D3',
      linkHoverColor: '#005EA2',
      separatorColor: '#AAB7B8',
      fontSize: 14,
    },
  },
};

/**
 * AWS-specific CSS variables for custom styling
 * Can be used in component CSS with var(--aws-*)
 */
export const awsCssVariables = {
  // AWS color palette
  '--aws-squid-ink': '#232F3E',
  '--aws-smile-orange': '#FF9900',
  '--aws-pacific-blue': '#0972D3',
  '--aws-dark-navy': '#16191F',
  '--aws-light-gray': '#F2F3F3',
  '--aws-border-gray': '#D5DBDB',
  '--aws-text-secondary': '#545B64',
  '--aws-text-tertiary': '#879596',

  // AWS spacing
  '--aws-space-xs': '4px',
  '--aws-space-s': '8px',
  '--aws-space-m': '12px',
  '--aws-space-l': '16px',
  '--aws-space-xl': '20px',
  '--aws-space-xxl': '24px',

  // AWS borders
  '--aws-border-width': '1px',
  '--aws-border-radius': '2px',
  '--aws-border-radius-large': '4px',

  // AWS shadows
  '--aws-shadow-default': '0 1px 1px 0 rgba(0, 28, 36, 0.3), 1px 1px 1px 0 rgba(0, 28, 36, 0.15), -1px 1px 1px 0 rgba(0, 28, 36, 0.15)',
  '--aws-shadow-sticky': '0 2px 4px 0 rgba(0, 28, 36, 0.5)',
  '--aws-shadow-dropdown': '0 4px 20px 1px rgba(0, 28, 36, 0.1)',
};
