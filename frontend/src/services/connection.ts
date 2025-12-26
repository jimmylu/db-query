import { axiosInstance } from './api';
import { DatabaseConnection, CreateConnectionRequest } from '../types';

export const connectionService = {
  /**
   * List all database connections
   */
  async listConnections(): Promise<DatabaseConnection[]> {
    const response = await axiosInstance.get<{ connections: DatabaseConnection[] }>('/connections');
    return response.data.connections;
  },

  /**
   * Create a new database connection
   */
  async createConnection(data: CreateConnectionRequest): Promise<{ connection: DatabaseConnection; metadata: any }> {
    const response = await axiosInstance.post<{ connection: DatabaseConnection; metadata: any }>(
      '/connections',
      data
    );
    return response.data;
  },

  /**
   * Get connection details by ID
   */
  async getConnection(id: string): Promise<DatabaseConnection> {
    const response = await axiosInstance.get<DatabaseConnection>(`/connections/${id}`);
    return response.data;
  },

  /**
   * Delete a connection
   */
  async deleteConnection(id: string): Promise<void> {
    await axiosInstance.delete(`/connections/${id}`);
  },
};

