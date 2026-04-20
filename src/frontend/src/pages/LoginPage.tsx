import { useState, type FormEvent } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { toast } from 'sonner';

import { Button } from '@/components/Button';
import { Input } from '@/components/Input';
import { useAuth } from '@/context/AuthContext';
import { ROUTES } from '@/lib/constants';

export function LoginPage() {
  const navigate = useNavigate();
  const { login, isLoading } = useAuth();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setError('');

    if (!email || !password) {
      setError('Email e senha são obrigatórios');
      return;
    }

    try {
      const result = await login(email, password);

      if ('challenge_token' in result) {
        navigate(ROUTES.TWO_FACTOR, { state: { challengeToken: result.challenge_token } });
      } else {
        navigate(ROUTES.DASHBOARD);
      }
    } catch (err: unknown) {
      const errorMessage = (err as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Credenciais inválidas';
      setError(errorMessage);
      toast.error(errorMessage);
    }
  }

  return (
    <div className="page-container">
      <div className="card">
        <div className="text-center mb-8">
          <h1 className="text-2xl font-bold text-slate-900">DEX Account</h1>
          <p className="text-slate-500 mt-1">Faça login para continuar</p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <Input
            type="email"
            name="email"
            label="Email"
            placeholder="seu@email.com"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            autoComplete="email"
            required
          />

          <Input
            type="password"
            name="password"
            label="Senha"
            placeholder="••••••••"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="current-password"
            required
          />

          {error && <p className="form-error text-center">{error}</p>}

          <Button type="submit" className="w-full" isLoading={isLoading}>
            Entrar
          </Button>
        </form>

        <div className="mt-6 text-center">
          <Link to={ROUTES.RECOVERY} className="text-sm text-brand-600 hover:text-brand-700">
            Esqueci minha senha
          </Link>
        </div>
      </div>
    </div>
  );
}
