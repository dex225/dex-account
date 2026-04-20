export const API_PATHS = {
  AUTH: {
    LOGIN: '/api/v1/auth/login',
    VERIFY_2FA: '/api/v1/auth/verify-2fa',
    REFRESH: '/api/v1/auth/refresh',
    LOGOUT: '/api/v1/auth/logout',
    PASSWORD_FORGOT: '/api/v1/auth/password/forgot',
    PASSWORD_RESET: '/api/v1/auth/password/reset',
    SETUP_2FA: '/api/v1/auth/2fa/setup',
    ENABLE_2FA: '/api/v1/auth/2fa/enable',
    EMERGENCY_RECOVER: '/api/v1/auth/emergency/recover',
  },
  USERS: {
    CREATE: '/api/v1/users/create',
    ME: '/api/v1/users/me',
  },
} as const;

export const ROUTES = {
  HOME: '/',
  TWO_FACTOR: '/2fa',
  RECOVERY: '/recovery',
  RESET: '/reset',
  DASHBOARD: '/dashboard',
} as const;

export const TOKEN_EXPIRY_SECONDS = 15 * 60;
export const CHALLENGE_EXPIRY_SECONDS = 5 * 60;
export const PASSWORD_RESET_EXPIRY_MINUTES = 30;
