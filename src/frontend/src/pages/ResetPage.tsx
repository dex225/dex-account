import { useState, useEffect, type FormEvent } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { toast } from 'sonner';

import { Button } from '@/components/Button';
import { Input } from '@/components/Input';
import { useAuth } from '@/context/AuthContext';
import { ROUTES, PASSWORD_RESET_EXPIRY_MINUTES } from '@/lib/constants';

export function ResetPage() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { resetPassword, isLoading } = useAuth();
  const [token, setToken] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [countdown, setCountdown] = useState(PASSWORD_RESET_EXPIRY_MINUTES * 60);

  const tokenFromUrl = searchParams.get('token') || '';

  useEffect(() => {
    if (!tokenFromUrl) {
      toast.error('Token inválido ou expirado');
      navigate(ROUTES.HOME);
      return;
    }
    setToken(tokenFromUrl);

    const timer = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(timer);
          toast.error('Token expirado. Solicite um novo link.');
          navigate(ROUTES.RECOVERY);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [tokenFromUrl, navigate]);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError('');

    if (password.length < 8) {
      setError('Senha deve ter pelo menos 8 caracteres');
      return;
    }

    if (password !== confirmPassword) {
      setError('As senhas não coincidem');
      return;
    }

    try {
      await resetPassword(token, password);
      toast.success('Senha redefinida com sucesso');
      navigate(ROUTES.HOME);
    } catch (err: unknown) {
      const errorMessage = (err as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Erro ao redefinir senha';
      setError(errorMessage);
      toast.error(errorMessage);
    }
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  return (
    <div className="page-container">
      <div className="card">
        <div className="text-center mb-8">
          <h1 className="text-2xl font-bold text-slate-900">Nova senha</h1>
          <p className="text-slate-500 mt-1">Crie uma nova senha para sua conta</p>
          <p className="text-sm text-slate-400 mt-2">
            Link expira em <span className="font-mono text-slate-600">{formatTime(countdown)}</span>
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <Input
            type="password"
            name="password"
            label="Nova senha"
            placeholder="••••••••"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="new-password"
            required
          />

          <Input
            type="password"
            name="confirmPassword"
            label="Confirmar senha"
            placeholder="••••••••"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            autoComplete="new-password"
            required
          />

          {error && <p className="form-error text-center">{error}</p>}

          <Button type="submit" className="w-full" isLoading={isLoading}>
            Redefinir senha
          </Button>
        </form>
      </div>
    </div>
  );
}
