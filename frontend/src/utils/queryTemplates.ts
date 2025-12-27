// Query Templates Management
// Save and load commonly used SQL query templates

const STORAGE_KEY = 'db-query-templates';
const MAX_TEMPLATES = 50;

export interface QueryTemplate {
  id: string;
  name: string;
  description?: string;
  query: string;
  category?: string;
  createdAt: number;
  lastUsed?: number;
  useCount: number;
}

export class QueryTemplateManager {
  /**
   * Get all saved templates
   */
  static getTemplates(): QueryTemplate[] {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (!stored) return this.getDefaultTemplates();

      const userTemplates = JSON.parse(stored);
      // Merge with default templates
      return [...this.getDefaultTemplates(), ...userTemplates];
    } catch (error) {
      console.error('Failed to load query templates:', error);
      return this.getDefaultTemplates();
    }
  }

  /**
   * Get default built-in templates
   */
  static getDefaultTemplates(): QueryTemplate[] {
    return [
      {
        id: 'default-select-all',
        name: '查询所有记录',
        description: '从表中查询所有列和行',
        query: 'SELECT * FROM table_name LIMIT 100',
        category: '基础查询',
        createdAt: Date.now(),
        useCount: 0,
      },
      {
        id: 'default-select-columns',
        name: '查询指定列',
        description: '从表中查询特定列',
        query: 'SELECT column1, column2, column3\nFROM table_name\nWHERE condition\nLIMIT 100',
        category: '基础查询',
        createdAt: Date.now(),
        useCount: 0,
      },
      {
        id: 'default-count',
        name: '统计记录数',
        description: '统计表中符合条件的记录总数',
        query: 'SELECT COUNT(*) as total\nFROM table_name\nWHERE condition',
        category: '聚合查询',
        createdAt: Date.now(),
        useCount: 0,
      },
      {
        id: 'default-group-by',
        name: '分组统计',
        description: '按字段分组并统计',
        query: 'SELECT column1, COUNT(*) as count\nFROM table_name\nGROUP BY column1\nORDER BY count DESC\nLIMIT 100',
        category: '聚合查询',
        createdAt: Date.now(),
        useCount: 0,
      },
      {
        id: 'default-inner-join',
        name: '内连接',
        description: '两个表的内连接查询',
        query: 'SELECT a.*, b.*\nFROM table1 a\nINNER JOIN table2 b ON a.id = b.table1_id\nLIMIT 100',
        category: '连接查询',
        createdAt: Date.now(),
        useCount: 0,
      },
      {
        id: 'default-left-join',
        name: '左连接',
        description: '两个表的左连接查询',
        query: 'SELECT a.*, b.*\nFROM table1 a\nLEFT JOIN table2 b ON a.id = b.table1_id\nLIMIT 100',
        category: '连接查询',
        createdAt: Date.now(),
        useCount: 0,
      },
      {
        id: 'default-date-range',
        name: '日期范围查询',
        description: '查询指定日期范围内的记录',
        query: 'SELECT *\nFROM table_name\nWHERE date_column >= \'2024-01-01\'\n  AND date_column < \'2024-02-01\'\nLIMIT 100',
        category: '条件查询',
        createdAt: Date.now(),
        useCount: 0,
      },
      {
        id: 'default-distinct',
        name: '去重查询',
        description: '查询不重复的值',
        query: 'SELECT DISTINCT column1, column2\nFROM table_name\nORDER BY column1\nLIMIT 100',
        category: '基础查询',
        createdAt: Date.now(),
        useCount: 0,
      },
    ];
  }

  /**
   * Add a new template
   */
  static addTemplate(template: Omit<QueryTemplate, 'id' | 'createdAt' | 'useCount'>): void {
    try {
      const templates = this.getUserTemplates();

      const newTemplate: QueryTemplate = {
        ...template,
        id: `user-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        createdAt: Date.now(),
        useCount: 0,
      };

      templates.unshift(newTemplate);

      // Limit number of user templates
      const trimmedTemplates = templates.slice(0, MAX_TEMPLATES);

      localStorage.setItem(STORAGE_KEY, JSON.stringify(trimmedTemplates));
    } catch (error) {
      console.error('Failed to save query template:', error);
      throw new Error('保存查询模板失败');
    }
  }

  /**
   * Update an existing template
   */
  static updateTemplate(id: string, updates: Partial<QueryTemplate>): void {
    try {
      const templates = this.getUserTemplates();
      const index = templates.findIndex(t => t.id === id);

      if (index === -1) {
        throw new Error('Template not found');
      }

      templates[index] = {
        ...templates[index],
        ...updates,
      };

      localStorage.setItem(STORAGE_KEY, JSON.stringify(templates));
    } catch (error) {
      console.error('Failed to update query template:', error);
      throw new Error('更新查询模板失败');
    }
  }

  /**
   * Delete a template
   */
  static deleteTemplate(id: string): void {
    try {
      const templates = this.getUserTemplates();
      const filtered = templates.filter(t => t.id !== id);

      localStorage.setItem(STORAGE_KEY, JSON.stringify(filtered));
    } catch (error) {
      console.error('Failed to delete query template:', error);
      throw new Error('删除查询模板失败');
    }
  }

  /**
   * Record template usage
   */
  static recordUsage(id: string): void {
    try {
      // Only track usage for user templates
      if (!id.startsWith('user-')) {
        return;
      }

      const templates = this.getUserTemplates();
      const template = templates.find(t => t.id === id);

      if (template) {
        template.useCount = (template.useCount || 0) + 1;
        template.lastUsed = Date.now();

        localStorage.setItem(STORAGE_KEY, JSON.stringify(templates));
      }
    } catch (error) {
      console.error('Failed to record template usage:', error);
    }
  }

  /**
   * Get user-created templates only
   */
  private static getUserTemplates(): QueryTemplate[] {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (!stored) return [];
      return JSON.parse(stored);
    } catch (error) {
      console.error('Failed to load user templates:', error);
      return [];
    }
  }

  /**
   * Clear all user templates (keep default templates)
   */
  static clearUserTemplates(): void {
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch (error) {
      console.error('Failed to clear templates:', error);
      throw new Error('清空查询模板失败');
    }
  }

  /**
   * Get templates by category
   */
  static getTemplatesByCategory(category: string): QueryTemplate[] {
    const templates = this.getTemplates();
    return templates.filter(t => t.category === category);
  }

  /**
   * Get all categories
   */
  static getCategories(): string[] {
    const templates = this.getTemplates();
    const categories = new Set<string>();

    templates.forEach(t => {
      if (t.category) {
        categories.add(t.category);
      }
    });

    return Array.from(categories).sort();
  }
}
