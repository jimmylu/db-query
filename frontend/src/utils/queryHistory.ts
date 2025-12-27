// Query History Management using localStorage

export interface QueryHistoryItem {
  id: string;
  query: string;
  connectionId: string;
  connectionName: string;
  timestamp: number;
  success: boolean;
  rowCount?: number;
  executionTimeMs?: number;
}

const STORAGE_KEY = 'db_query_history';
const MAX_HISTORY_SIZE = 50; // Keep last 50 queries

export class QueryHistoryManager {
  static getHistory(): QueryHistoryItem[] {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (!stored) return [];
      return JSON.parse(stored);
    } catch (error) {
      console.error('Failed to load query history:', error);
      return [];
    }
  }

  static addQuery(item: Omit<QueryHistoryItem, 'id' | 'timestamp'>): void {
    try {
      const history = this.getHistory();

      const newItem: QueryHistoryItem = {
        ...item,
        id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        timestamp: Date.now(),
      };

      // Add to beginning of array (most recent first)
      history.unshift(newItem);

      // Keep only last MAX_HISTORY_SIZE queries
      const trimmedHistory = history.slice(0, MAX_HISTORY_SIZE);

      localStorage.setItem(STORAGE_KEY, JSON.stringify(trimmedHistory));
    } catch (error) {
      console.error('Failed to save query to history:', error);
    }
  }

  static clearHistory(): void {
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch (error) {
      console.error('Failed to clear query history:', error);
    }
  }

  static deleteQuery(id: string): void {
    try {
      const history = this.getHistory();
      const filtered = history.filter(item => item.id !== id);
      localStorage.setItem(STORAGE_KEY, JSON.stringify(filtered));
    } catch (error) {
      console.error('Failed to delete query from history:', error);
    }
  }

  static getRecentQueries(limit: number = 10): QueryHistoryItem[] {
    return this.getHistory().slice(0, limit);
  }

  static getQueriesByConnection(connectionId: string, limit: number = 10): QueryHistoryItem[] {
    const history = this.getHistory();
    return history
      .filter(item => item.connectionId === connectionId)
      .slice(0, limit);
  }

  static searchQueries(searchText: string, limit: number = 20): QueryHistoryItem[] {
    const history = this.getHistory();
    const lowerSearch = searchText.toLowerCase();
    return history
      .filter(item => item.query.toLowerCase().includes(lowerSearch))
      .slice(0, limit);
  }
}
