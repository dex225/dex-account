import axios, { type AxiosInstance, type InternalAxiosRequestConfig } from 'axios';

import { API_PATHS, ROUTES } from './constants';

export interface LoginResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
}

export interface TwoFactorChallengeResponse {
  challenge_token: string;
  expires_in: number;
}

export interface User {
  id: string;
  email: string;
  is_2fa_enabled: boolean;
  created_at: string;
}

export interface ApiError {
  error: string;
}

const api: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_TARGET || 'http://localhost:3000',
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
  },
});

let accessToken: string | null = null;
let refreshPromise: Promise<string | null> | null = null;

export function setAccessToken(token: string | null): void {
  accessToken = token;
}

export function getAccessToken(): string | null {
  return accessToken;
}

api.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    if (accessToken && config.headers) {
      config.headers.Authorization = `Bearer ${accessToken}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config;

    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;

      if (!refreshPromise) {
        refreshPromise = refreshAccessToken()
          .finally(() => {
            refreshPromise = null;
          });
      }

      const newToken = await refreshPromise;

      if (newToken) {
        setAccessToken(newToken);
        originalRequest.headers.Authorization = `Bearer ${newToken}`;
        return api(originalRequest);
      } else {
        setAccessToken(null);
        window.location.href = ROUTES.HOME;
        return Promise.reject(error);
      }
    }

    return Promise.reject(error);
  }
);

export async function refreshAccessToken(): Promise<string | null> {
  try {
    const response = await axios.post(
      `${import.meta.env.VITE_API_TARGET || 'http://localhost:3000'}${API_PATHS.AUTH.REFRESH}`,
      {},
      { withCredentials: true }
    );
    return response.data.access_token;
  } catch {
    return null;
  }
}

export async function login(email: string, password: string): Promise<LoginResponse | TwoFactorChallengeResponse> {
  const response = await api.post(API_PATHS.AUTH.LOGIN, { email, password });
  return response.data;
}

export async function verify2FA(challengeToken: string, code: string): Promise<LoginResponse> {
  const response = await api.post(API_PATHS.AUTH.VERIFY_2FA, { challenge_token: challengeToken, code });
  return response.data;
}

export async function logout(): Promise<void> {
  await api.post(API_PATHS.AUTH.LOGOUT);
  setAccessToken(null);
}

export async function forgotPassword(email: string): Promise<{ message: string }> {
  const response = await api.post(API_PATHS.AUTH.PASSWORD_FORGOT, { email });
  return response.data;
}

export async function resetPassword(token: string, newPassword: string): Promise<{ message: string }> {
  const response = await api.post(API_PATHS.AUTH.PASSWORD_RESET, { token, new_password: newPassword });
  return response.data;
}

export async function setup2FA(): Promise<{ totp_uri: string; secret: string }> {
  const response = await api.post(API_PATHS.AUTH.SETUP_2FA);
  return response.data;
}

export async function enable2FA(code: string): Promise<{ message: string }> {
  const response = await api.post(API_PATHS.AUTH.ENABLE_2FA, { code });
  return response.data;
}

export async function getMe(): Promise<User> {
  const response = await api.get(API_PATHS.USERS.ME);
  return response.data;
}

export default api;
