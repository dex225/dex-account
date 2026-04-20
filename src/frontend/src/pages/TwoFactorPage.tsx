import { useState, useEffect, type FormEvent } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { toast } from 'sonner';

import { Button } from '@/components/Button';
import { Input } from '@/components/Input';
import { useAuth } from '@/context/AuthContext';
import { ROUTES, CHALLENGE_EXPIRY_SECONDS } from '@/lib/constants';

export function TwoFactorPage() {
  const navigate = useNavigate();
  const location = useLocation();
  const { verify2FA, isLoading } = useAuth();
  const [code, setCode] = useState('');
  const [error, setError] = useState('');
  const [countdown, setCountdown] = useState(CHALLENGE_EXPIRY_SECONDS);

  const challengeToken = location.state?.challengeToken as string;

  useEffect(() => {
    if (!challengeToken) {
      navigate(ROUTES.HOME);
      return;
    }

    const timer = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(timer);
          toast.error('Código expirado. Faça login novamente.');
          navigate(ROUTES.HOME);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [challengeToken, navigate]);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError('');

    if (!code || code.length !== 6) {
      setError('Código deve ter 6 dígitos');
      return;
    }

    try {
      await verify2FA(challengeToken, code);
      toast.success('Login realizado com sucesso');
    } catch (err: unknown) {
      const errorMessage = (err as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Código inválido';
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
          <h1 className="text-2xl font-bold text-slate-900">Verificação em duas etapas</h1>
          <p className="text-slate-500 mt-1">Digite o código do seu autenticador</p>
          <p className="text-sm text-slate-400 mt-2">
            Código expira em <span className="font-mono text-slate-600">{formatTime(countdown)}</span>
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <Input
            type="text"
            name="code"
            placeholder="000000"
            value={code}
            onChange={(e) => setCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
            autoComplete="one-time-code"
            inputMode="numeric"
            pattern="[0-9]*"
            maxLength={6}
            className="text-center text-2xl tracking-widest font-mono"
            autoFocus
            required
          />

          {error && <p className="form-error text-center">{error}</p>}

          <Button type="submit" className="w-full" isLoading={isLoading}>
            Verificar
          </Button>
        </form>

        <div className="mt-6 text-center">
          <button
            type="button"
            onClick={() => navigate(ROUTES.HOME)}
            className="text-sm text-slate-500 hover:text-slate-700"
          >
            Voltar ao login
          </button>
        </div>
      </div>
    </div>
  );
}
