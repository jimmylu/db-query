import { axiosInstance } from './api';
import { DatabaseMetadata } from '../types';

export const metadataService = {
  /**
   * Get database metadata for a connection
   * @param connectionId - The connection ID
   * @param refresh - Whether to force refresh from database
   */
  async getMetadata(connectionId: string, refresh: boolean = false): Promise<{ metadata: DatabaseMetadata; cached: boolean }> {
    const params = refresh ? { refresh: 'true' } : {};
    const response = await axiosInstance.get<{ metadata: DatabaseMetadata; cached: boolean }>(
      `/connections/${connectionId}/metadata`,
      { params }
    );
    return response.data;
  },
};

