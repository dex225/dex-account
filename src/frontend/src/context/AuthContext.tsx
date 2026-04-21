import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';

import {
  login as apiLogin,
  verify2FA as apiVerify2FA,
  logout as apiLogout,
  forgotPassword as apiForgotPassword,
  resetPassword as apiResetPassword,
  setup2FA as apiSetup2FA,
  enable2FA as apiEnable2FA,
  getMe as apiGetMe,
  setAccessToken,
  refreshAccessToken,
  type LoginResponse,
  type TwoFactorChallengeResponse,
  type User,
} from '@/lib/api';
import { ROUTES } from '@/lib/constants';

interface AuthState {
  accessToken: string | null;
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

interface AuthContextType extends AuthState {
  login: (email: string, password: string) => Promise<LoginResponse | TwoFactorChallengeResponse>;
  verify2FA: (challengeToken: string, code: string) => Promise<LoginResponse>;
  logout: () => Promise<void>;
  forgotPassword: (email: string) => Promise<{ message: string }>;
  resetPassword: (token: string, newPassword: string) => Promise<{ message: string }>;
  setup2FA: () => Promise<{ totp_uri: string; secret: string }>;
  enable2FA: (code: string) => Promise<{ message: string }>;
  refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const navigate = useNavigate();
  const [state, setState] = useState<AuthState>({
    accessToken: null,
    user: null,
    isAuthenticated: false,
    isLoading: true,
  });

  useEffect(() => {
    refreshAccessToken()
      .then(async (token) => {
        if (token) {
          setAccessToken(token);
          const user = await apiGetMe();
          setState({
            accessToken: token,
            user,
            isAuthenticated: true,
            isLoading: false,
          });
        } else {
          setState((prev) => ({ ...prev, isLoading: false }));
        }
      })
      .catch(() => {
        setState((prev) => ({ ...prev, isLoading: false }));
      });
  }, []);

  const login = useCallback(async (email: string, password: string) => {
    setState((prev) => ({ ...prev, isLoading: true }));
    try {
      const result = await apiLogin(email, password);
      if ('access_token' in result) {
        setAccessToken(result.access_token);
        setState({
          accessToken: result.access_token,
          user: null,
          isAuthenticated: true,
          isLoading: false,
        });
      } else {
        setState((prev) => ({ ...prev, isLoading: false }));
      }
      return result;
    } catch (error) {
      setState((prev) => ({ ...prev, isLoading: false }));
      throw error;
    }
  }, []);

  const verify2FA = useCallback(async (challengeToken: string, code: string) => {
    setState((prev) => ({ ...prev, isLoading: true }));
    try {
      const result = await apiVerify2FA(challengeToken, code);
      setAccessToken(result.access_token);
      const user = await apiGetMe();
      setState({
        accessToken: result.access_token,
        user,
        isAuthenticated: true,
        isLoading: false,
      });
      navigate(ROUTES.DASHBOARD);
      return result;
    } catch (error) {
      setState((prev) => ({ ...prev, isLoading: false }));
      throw error;
    }
  }, [navigate]);

  const logout = useCallback(async () => {
    try {
      await apiLogout();
    } finally {
      setAccessToken(null);
      setState({
        accessToken: null,
        user: null,
        isAuthenticated: false,
        isLoading: false,
      });
      navigate(ROUTES.HOME);
    }
  }, [navigate]);

  const forgotPassword = useCallback(async (email: string) => {
    return apiForgotPassword(email);
  }, []);

  const resetPassword = useCallback(async (token: string, newPassword: string) => {
    return apiResetPassword(token, newPassword);
  }, []);

  const setup2FA = useCallback(async () => {
    return apiSetup2FA();
  }, []);

  const enable2FA = useCallback(async (code: string) => {
    const result = await apiEnable2FA(code);
    await refreshUser();
    return result;
  }, []);

  const refreshUser = useCallback(async () => {
    try {
      const user = await apiGetMe();
      setState((prev) => ({ ...prev, user }));
    } catch {
      setState({
        accessToken: null,
        user: null,
        isAuthenticated: false,
        isLoading: false,
      });
    }
  }, []);

  return (
    <AuthContext.Provider
      value={{
        ...state,
        login,
        verify2FA,
        logout,
        forgotPassword,
        resetPassword,
        setup2FA,
        enable2FA,
        refreshUser,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextType {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
