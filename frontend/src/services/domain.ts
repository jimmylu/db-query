import { axiosInstance } from './api';
import {
  DomainResponse,
  CreateDomainRequest,
  UpdateDomainRequest,
  DatabaseConnection
} from '../types';

export const domainService = {
  /**
   * List all domains with resource counts
   */
  async listDomains(): Promise<DomainResponse[]> {
    const response = await axiosInstance.get<{ domains: DomainResponse[] }>('/domains');
    return response.data.domains;
  },

  /**
   * Get domain details by ID
   */
  async getDomain(id: string): Promise<DomainResponse> {
    const response = await axiosInstance.get<{ domain: DomainResponse }>(`/domains/${id}`);
    return response.data.domain;
  },

  /**
   * Create a new domain
   */
  async createDomain(data: CreateDomainRequest): Promise<DomainResponse> {
    const response = await axiosInstance.post<{ domain: DomainResponse }>('/domains', data);
    return response.data.domain;
  },

  /**
   * Update an existing domain
   */
  async updateDomain(id: string, data: UpdateDomainRequest): Promise<DomainResponse> {
    const response = await axiosInstance.put<{ domain: DomainResponse }>(`/domains/${id}`, data);
    return response.data.domain;
  },

  /**
   * Delete a domain (CASCADE deletes associated connections)
   */
  async deleteDomain(id: string): Promise<void> {
    await axiosInstance.delete(`/domains/${id}`);
  },

  /**
   * List connections for a specific domain
   */
  async listDomainConnections(id: string): Promise<DatabaseConnection[]> {
    const response = await axiosInstance.get<{ connections: DatabaseConnection[] }>(`/domains/${id}/connections`);
    return response.data.connections;
  },
};
