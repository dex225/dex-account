import { useState, type FormEvent } from 'react';
import { Link } from 'react-router-dom';
import { toast } from 'sonner';

import { Button } from '@/components/Button';
import { Input } from '@/components/Input';
import { useAuth } from '@/context/AuthContext';
import { ROUTES } from '@/lib/constants';

export function RecoveryPage() {
  const { forgotPassword, isLoading } = useAuth();
  const [email, setEmail] = useState('');
  const [submitted, setSubmitted] = useState(false);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();

    if (!email) return;

    try {
      await forgotPassword(email);
      setSubmitted(true);
      toast.success('Se o email existir, um link de recuperação foi enviado');
    } catch {
      toast.error('Erro ao enviar email de recuperação');
    }
  }

  if (submitted) {
    return (
      <div className="page-container">
        <div className="card">
          <div className="text-center">
            <div className="w-12 h-12 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-6 h-6 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h2 className="text-xl font-semibold text-slate-900 mb-2">Email enviado</h2>
            <p className="text-slate-500">
              Se o email <span className="font-medium">{email}</span> existir em nossa base, você receberá um link para redefinir sua senha.
            </p>
          </div>
          <div className="mt-6 text-center">
            <Link to={ROUTES.HOME} className="text-brand-600 hover:text-brand-700 font-medium">
              Voltar ao login
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="card">
        <div className="text-center mb-8">
          <h1 className="text-2xl font-bold text-slate-900">Recuperar senha</h1>
          <p className="text-slate-500 mt-1">Digite seu email para receber um link</p>
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

          <Button type="submit" className="w-full" isLoading={isLoading}>
            Enviar link de recuperação
          </Button>
        </form>

        <div className="mt-6 text-center">
          <Link to={ROUTES.HOME} className="text-sm text-slate-500 hover:text-slate-700">
            Voltar ao login
          </Link>
        </div>
      </div>
    </div>
  );
}
