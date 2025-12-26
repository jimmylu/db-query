import { DataProvider } from '@refinedev/core';
import { axiosInstance } from '../services/api';

export const dataProvider: DataProvider = {
  getList: async ({ resource }) => {
    const url = `/${resource}`;

    const { data } = await axiosInstance.get(url);

    return {
      data: data[resource] || [],
      total: data[resource]?.length || 0,
    };
  },

  getOne: async ({ resource, id }) => {
    const url = `/${resource}/${id}`;

    const { data } = await axiosInstance.get(url);

    return {
      data: data[resource] || data,
    };
  },

  create: async ({ resource, variables }) => {
    const url = `/${resource}`;

    const { data } = await axiosInstance.post(url, variables);

    return {
      data: data[resource] || data,
    };
  },

  update: async ({ resource, id, variables }) => {
    const url = `/${resource}/${id}`;

    const { data } = await axiosInstance.put(url, variables);

    return {
      data: data[resource] || data,
    };
  },

  deleteOne: async ({ resource, id }) => {
    const url = `/${resource}/${id}`;

    const { data } = await axiosInstance.delete(url);

    return {
      data: data[resource] || data,
    };
  },

  getApiUrl: () => {
    return import.meta.env.VITE_API_URL || 'http://localhost:3000/api';
  },
};

