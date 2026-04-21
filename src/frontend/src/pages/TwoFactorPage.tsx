import { useState, type FormEvent } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { toast } from 'sonner';

import { Button } from '@/components/Button';
import { Input } from '@/components/Input';
import { useAuth } from '@/context/AuthContext';
import { ROUTES } from '@/lib/constants';

export function TwoFactorPage() {
  const navigate = useNavigate();
  const location = useLocation();
  const { verify2FA, isLoading } = useAuth();
  const [code, setCode] = useState('');
  const [error, setError] = useState('');

  const challengeToken = location.state?.challengeToken as string;

  if (!challengeToken) {
    navigate(ROUTES.HOME);
    return null;
  }

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

  return (
    <div className="page-container">
      <div className="card">
        <div className="text-center mb-8">
          <h1 className="text-2xl font-bold text-slate-900">Verificação em duas etapas</h1>
          <p className="text-slate-500 mt-1">Digite o código do seu autenticador</p>
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
