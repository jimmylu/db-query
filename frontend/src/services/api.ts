import axios, { AxiosInstance, AxiosError } from 'axios';

// Create axios instance with default configuration
// In development, use relative path to leverage Vite proxy
// In production, use full URL or environment variable
const baseURL = import.meta.env.VITE_API_URL || (import.meta.env.DEV ? '/api' : 'http://localhost:3000/api');

const apiClient: AxiosInstance = axios.create({
  baseURL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor
apiClient.interceptors.request.use(
  (config) => {
    // Add any auth tokens or headers here if needed
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor for error handling
apiClient.interceptors.response.use(
  (response) => {
    return response;
  },
  (error: AxiosError) => {
    // Handle common errors
    if (error.response) {
      // Server responded with error status
      const status = error.response.status;
      const data = error.response.data as { error?: { code: string; message: string; details?: string } };

      switch (status) {
        case 400:
          console.error('Bad Request:', data.error?.message || error.message);
          break;
        case 404:
          console.error('Not Found:', data.error?.message || error.message);
          break;
        case 500:
          console.error('Server Error:', data.error?.message || error.message);
          break;
        default:
          console.error('API Error:', error.message);
      }
    } else if (error.request) {
      // Request was made but no response received
      console.error('Network Error: No response received');
    } else {
      // Something else happened
      console.error('Error:', error.message);
    }

    return Promise.reject(error);
  }
);

export const axiosInstance = apiClient;
export default apiClient;

// Export error types for use in components
export interface ApiError {
  code: string;
  message: string;
  details?: string;
}

export const extractApiError = (error: unknown): ApiError => {
  if (axios.isAxiosError(error)) {
    const data = error.response?.data as { error?: ApiError };
    if (data?.error) {
      return data.error;
    }
  }
  return {
    code: 'UNKNOWN_ERROR',
    message: error instanceof Error ? error.message : 'An unknown error occurred',
  };
};

