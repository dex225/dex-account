import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { QRCodeSVG } from 'qrcode.react';

import { Button } from '@/components/Button';
import { Input } from '@/components/Input';
import { Spinner } from '@/components/Spinner';
import { useAuth } from '@/context/AuthContext';

export function DashboardPage() {
  const { user, logout, setup2FA, enable2FA, isLoading, refreshUser } = useAuth();
  const [showSetup2FA, setShowSetup2FA] = useState(false);
  const [totpData, setTotpData] = useState<{ totp_uri: string; secret: string } | null>(null);
  const [setupCode, setSetupCode] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    refreshUser();
  }, [refreshUser]);

  async function handleSetup2FA() {
    try {
      const data = await setup2FA();
      setTotpData(data);
      setShowSetup2FA(true);
    } catch {
      toast.error('Erro ao gerar código 2FA');
    }
  }

  async function handleEnable2FA() {
    setError('');
    if (setupCode.length !== 6) {
      setError('Código deve ter 6 dígitos');
      return;
    }

    try {
      await enable2FA(setupCode);
      toast.success('2FA ativado com sucesso');
      setShowSetup2FA(false);
      setTotpData(null);
      setSetupCode('');
    } catch (err: unknown) {
      const errorMessage = (err as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Código inválido';
      setError(errorMessage);
      toast.error(errorMessage);
    }
  }

  async function handleLogout() {
    try {
      await logout();
      toast.success('Logout realizado');
    } catch {
      toast.error('Erro ao fazer logout');
    }
  }

  if (!user) {
    return (
      <div className="page-container">
        <Spinner size="lg" />
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="card">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-xl font-bold text-slate-900">Dashboard</h1>
          <Button variant="secondary" size="sm" onClick={handleLogout} isLoading={isLoading}>
            Sair
          </Button>
        </div>

        <div className="space-y-6">
          <div className="p-4 bg-slate-50 rounded-lg">
            <p className="text-sm text-slate-500">Email</p>
            <p className="font-medium text-slate-900">{user.email}</p>
          </div>

          <div className="p-4 bg-slate-50 rounded-lg">
            <p className="text-sm text-slate-500">Membro desde</p>
            <p className="font-medium text-slate-900">
              {new Date(user.created_at).toLocaleDateString('pt-BR')}
            </p>
          </div>

          <div className="p-4 bg-slate-50 rounded-lg">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-slate-500">Autenticação em duas etapas</p>
                <p className="font-medium text-slate-900">
                  {user.is_2fa_enabled ? 'Ativada' : 'Desativada'}
                </p>
              </div>
              {!user.is_2fa_enabled && !showSetup2FA && (
                <Button size="sm" onClick={handleSetup2FA} isLoading={isLoading}>
                  Ativar 2FA
                </Button>
              )}
            </div>
          </div>

          {showSetup2FA && totpData && (
            <div className="p-4 border border-brand-200 bg-brand-50 rounded-lg space-y-4">
              <div className="text-center">
                <p className="text-sm font-medium text-slate-700 mb-3">Escaneie o QR Code com seu autenticador</p>
                <div className="inline-block p-4 bg-white rounded-lg">
                  <QRCodeSVG value={totpData.totp_uri} size={180} level="M" />
                </div>
              </div>

              <div className="text-center">
                <p className="text-xs text-slate-500 mb-1">Ou digite este código manualmente:</p>
                <p className="font-mono text-sm text-slate-700 break-all">{totpData.secret}</p>
              </div>

              <div className="space-y-3">
                <Input
                  type="text"
                  name="setupCode"
                  placeholder="000000"
                  value={setupCode}
                  onChange={(e) => setSetupCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                  inputMode="numeric"
                  pattern="[0-9]*"
                  maxLength={6}
                  className="text-center text-xl tracking-widest font-mono"
                  autoFocus
                />

                {error && <p className="form-error text-center">{error}</p>}

                <div className="flex gap-2">
                  <Button variant="secondary" className="flex-1" onClick={() => setShowSetup2FA(false)}>
                    Cancelar
                  </Button>
                  <Button className="flex-1" onClick={handleEnable2FA} isLoading={isLoading}>
                    Confirmar
                  </Button>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
