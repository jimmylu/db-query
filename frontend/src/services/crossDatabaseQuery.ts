import apiClient from './api';
import type {
  CrossDatabaseQueryRequest,
  CrossDatabaseQueryResponse,
} from '../types/cross-database';

export const crossDatabaseQueryService = {
  /**
   * Execute a cross-database query
   */
  async executeQuery(
    request: CrossDatabaseQueryRequest
  ): Promise<CrossDatabaseQueryResponse> {
    const response = await apiClient.post<CrossDatabaseQueryResponse>(
      '/cross-database/query',
      request
    );
    return response.data;
  },

  /**
   * Validate cross-database query syntax
   */
  async validateQuery(query: string): Promise<{ valid: boolean; error?: string }> {
    try {
      // Simple client-side validation
      if (!query.trim()) {
        return { valid: false, error: '查询不能为空' };
      }

      // Check for basic SQL injection patterns
      const dangerousPatterns = /\b(DROP|DELETE|UPDATE|INSERT|ALTER|CREATE|TRUNCATE|EXEC)\b/i;
      if (dangerousPatterns.test(query)) {
        return { valid: false, error: '仅支持 SELECT 查询' };
      }

      return { valid: true };
    } catch (error: any) {
      return { valid: false, error: error.message };
    }
  },

  /**
   * Generate sample cross-database queries
   */
  getSampleQueries(): Array<{ title: string; query: string; description: string }> {
    return [
      {
        title: '简单 JOIN',
        query: `SELECT u.username, t.title
FROM db1.users u
JOIN db2.todos t ON u.id = t.user_id
LIMIT 10`,
        description: '跨数据库关联查询用户和待办事项',
      },
      {
        title: 'JOIN with WHERE',
        query: `SELECT u.username, t.title, t.status
FROM db1.users u
JOIN db2.todos t ON u.id = t.user_id
WHERE t.status = 'pending'
LIMIT 10`,
        description: '带条件的跨数据库关联查询',
      },
      {
        title: '多列 JOIN',
        query: `SELECT u.id, u.username, u.email, t.title, t.priority
FROM db1.users u
JOIN db2.todos t ON u.id = t.user_id
ORDER BY t.priority DESC
LIMIT 20`,
        description: '多列选择的跨数据库查询',
      },
      {
        title: 'UNION（框架就绪）',
        query: `SELECT username FROM db1.users
UNION
SELECT title FROM db2.todos
LIMIT 15`,
        description: 'UNION 查询（当前返回 NOT_IMPLEMENTED）',
      },
    ];
  },
};
