import { axiosInstance } from './api';

/**
 * Database types supported by the unified SQL semantic layer
 */
export enum DatabaseType {
  PostgreSQL = 'postgresql',
  MySQL = 'mysql',
  Doris = 'doris',
  Druid = 'druid',
}

/**
 * Unified query request
 * Uses DataFusion SQL syntax that will be automatically translated to target database dialect
 */
export interface UnifiedQueryRequest {
  /** SQL query in DataFusion syntax */
  query: string;
  /** Target database type */
  database_type: DatabaseType;
  /** Query timeout in seconds (default: 30) */
  timeout_secs?: number;
  /** Whether to automatically apply LIMIT if not present (default: true) */
  apply_limit?: boolean;
  /** LIMIT value to apply (default: 1000) */
  limit_value?: number;
}

/**
 * Unified query response
 * Contains original query, translated query, and execution results
 */
export interface UnifiedQueryResponse {
  /** Original query in DataFusion SQL syntax */
  original_query: string;
  /** Translated query in target database dialect */
  translated_query: string;
  /** Target database type */
  database_type: DatabaseType;
  /** Query results as JSON objects */
  results: Record<string, any>[];
  /** Number of rows returned */
  row_count: number;
  /** Execution time in milliseconds */
  execution_time_ms: number;
  /** Whether LIMIT was automatically applied */
  limit_applied: boolean;
  /** Execution timestamp */
  executed_at: string;
}

/**
 * Unified Query Service
 *
 * Provides API methods for executing SQL queries using DataFusion's unified SQL syntax.
 * Queries are automatically translated to the target database's dialect.
 *
 * @example
 * ```typescript
 * // Execute a DataFusion SQL query against PostgreSQL
 * const response = await unifiedQueryService.executeUnifiedQuery(
 *   'conn-123',
 *   {
 *     query: "SELECT * FROM users WHERE created_at >= CURRENT_DATE - INTERVAL '7' DAY",
 *     database_type: DatabaseType.PostgreSQL,
 *     timeout_secs: 30,
 *     apply_limit: true,
 *     limit_value: 1000
 *   }
 * );
 *
 * console.log('Original:', response.original_query);
 * console.log('Translated:', response.translated_query);
 * console.log('Results:', response.results);
 * ```
 */
export const unifiedQueryService = {
  /**
   * Execute a unified SQL query using DataFusion semantic layer
   *
   * @param connectionId - Database connection ID
   * @param request - Unified query request with DataFusion SQL
   * @returns Unified query response with translation details and results
   *
   * @throws {Error} If database type doesn't match connection or query validation fails
   */
  async executeUnifiedQuery(
    connectionId: string,
    request: UnifiedQueryRequest
  ): Promise<UnifiedQueryResponse> {
    const response = await axiosInstance.post<UnifiedQueryResponse>(
      `/connections/${connectionId}/unified-query`,
      {
        query: request.query,
        database_type: request.database_type,
        timeout_secs: request.timeout_secs ?? 30,
        apply_limit: request.apply_limit ?? true,
        limit_value: request.limit_value ?? 1000,
      }
    );
    return response.data;
  },

  /**
   * Get database type from connection
   * Helper method to determine which DatabaseType to use for a connection
   *
   * @param connectionType - Connection type string from backend
   * @returns Corresponding DatabaseType enum value
   */
  getDatabaseType(connectionType: string): DatabaseType {
    const normalized = connectionType.toLowerCase();
    switch (normalized) {
      case 'postgresql':
      case 'postgres':
        return DatabaseType.PostgreSQL;
      case 'mysql':
      case 'mariadb':
        return DatabaseType.MySQL;
      case 'doris':
        return DatabaseType.Doris;
      case 'druid':
        return DatabaseType.Druid;
      default:
        throw new Error(`Unsupported database type: ${connectionType}`);
    }
  },

  /**
   * Check if unified query is supported for a database type
   *
   * @param databaseType - Database type to check
   * @returns true if unified queries are supported
   */
  isUnifiedQuerySupported(databaseType: DatabaseType): boolean {
    // PostgreSQL and MySQL fully supported
    // Doris and Druid support coming in Phase 5
    return databaseType === DatabaseType.PostgreSQL || databaseType === DatabaseType.MySQL;
  },

  /**
   * Get human-readable database type name
   *
   * @param databaseType - Database type enum value
   * @returns Display name for the database type
   */
  getDatabaseTypeName(databaseType: DatabaseType): string {
    switch (databaseType) {
      case DatabaseType.PostgreSQL:
        return 'PostgreSQL';
      case DatabaseType.MySQL:
        return 'MySQL';
      case DatabaseType.Doris:
        return 'Apache Doris';
      case DatabaseType.Druid:
        return 'Apache Druid';
      default:
        return databaseType;
    }
  },
};
