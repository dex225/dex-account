import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';

import { AuthProvider, useAuth } from '@/context/AuthContext';
import { Toaster } from '@/components/Toast';
import { LoginPage } from '@/pages/LoginPage';
import { TwoFactorPage } from '@/pages/TwoFactorPage';
import { RecoveryPage } from '@/pages/RecoveryPage';
import { ResetPage } from '@/pages/ResetPage';
import { DashboardPage } from '@/pages/DashboardPage';
import { ROUTES } from '@/lib/constants';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuth();
  return isAuthenticated ? <>{children}</> : <Navigate to={ROUTES.HOME} replace />;
}

function PublicRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuth();
  return isAuthenticated ? <Navigate to={ROUTES.DASHBOARD} replace /> : <>{children}</>;
}

function AppRoutes() {
  const { isAuthenticated } = useAuth();

  return (
    <Routes>
      <Route
        path={ROUTES.HOME}
        element={
          <PublicRoute>
            <LoginPage />
          </PublicRoute>
        }
      />
      <Route path={ROUTES.TWO_FACTOR} element={<TwoFactorPage />} />
      <Route path={ROUTES.RECOVERY} element={<RecoveryPage />} />
      <Route path={ROUTES.RESET} element={<ResetPage />} />
      <Route
        path={ROUTES.DASHBOARD}
        element={
          <ProtectedRoute>
            <DashboardPage />
          </ProtectedRoute>
        }
      />
      <Route path="*" element={<Navigate to={isAuthenticated ? ROUTES.DASHBOARD : ROUTES.HOME} replace />} />
    </Routes>
  );
}

export function App() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <AppRoutes />
        <Toaster position="top-center" richColors />
      </AuthProvider>
    </BrowserRouter>
  );
}
