import { axiosInstance } from './api';
import { QueryResult } from '../types';

export interface QueryRequest {
  query: string;
}

export interface NaturalLanguageQueryRequest {
  question: string;
}

export interface QueryResponse {
  query: QueryResult;
  generated_sql?: string;
}

export const queryService = {
  /**
   * Execute SQL query
   */
  async executeQuery(connectionId: string, query: string): Promise<QueryResponse> {
    const response = await axiosInstance.post<QueryResponse>(
      `/connections/${connectionId}/query`,
      { query }
    );
    return response.data;
  },

  /**
   * Execute natural language query
   */
  async executeNaturalLanguageQuery(
    connectionId: string,
    question: string
  ): Promise<QueryResponse> {
    const response = await axiosInstance.post<QueryResponse>(
      `/connections/${connectionId}/nl-query`,
      { question }
    );
    return response.data;
  },
};

