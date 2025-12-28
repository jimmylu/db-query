import api from './api';
import type {
  SavedQuery,
  CreateSavedQueryRequest,
  UpdateSavedQueryRequest,
  QueryHistory,
} from '../types';

// Saved Query API functions
export const listSavedQueries = async (domainId: string): Promise<SavedQuery[]> => {
  const response = await api.get(`/domains/${domainId}/queries/saved`);
  return response.data;
};

export const getSavedQuery = async (domainId: string, queryId: string): Promise<SavedQuery> => {
  const response = await api.get(`/domains/${domainId}/queries/saved/${queryId}`);
  return response.data;
};

export const createSavedQuery = async (
  domainId: string,
  request: CreateSavedQueryRequest
): Promise<SavedQuery> => {
  const response = await api.post(`/domains/${domainId}/queries/saved`, request);
  return response.data;
};

export const updateSavedQuery = async (
  domainId: string,
  queryId: string,
  request: UpdateSavedQueryRequest
): Promise<SavedQuery> => {
  const response = await api.put(`/domains/${domainId}/queries/saved/${queryId}`, request);
  return response.data;
};

export const deleteSavedQuery = async (domainId: string, queryId: string): Promise<void> => {
  await api.delete(`/domains/${domainId}/queries/saved/${queryId}`);
};

// Query History API functions
export const listQueryHistory = async (
  domainId: string,
  limit: number = 50
): Promise<QueryHistory[]> => {
  const response = await api.get(`/domains/${domainId}/queries/history`, {
    params: { limit },
  });
  return response.data;
};

export const listConnectionQueryHistory = async (
  domainId: string,
  connectionId: string,
  limit: number = 50
): Promise<QueryHistory[]> => {
  const response = await api.get(
    `/domains/${domainId}/connections/${connectionId}/history`,
    {
      params: { limit },
    }
  );
  return response.data;
};
