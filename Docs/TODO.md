# Tarefas - DEX Account

## Status Atual

### ✅ Backend - PRODUÇÃO
- Login/logout com JWT + RTR (Refresh Token Rotation)
- 2FA com TOTP
- Recuperação de senha
- Recuperação de emergência
- Health checks (/health, /ready)
- Rate limiting com SmartIpKeyExtractor (funciona com Traefik)
- Prometheus metrics exporter (porta 3001)
- Migrations automáticas
- Docker multi-stage build
- Cleanup automático de tokens expirados
- Tracing/logging
- **curl adicionado ao container** para healthcheck funcionar

### ✅ Frontend - PRONTO PARA DEPLOY
- Setup: Vite + React + TypeScript
- Estilização com Tailwind CSS
- HTTP client com Axios + interceptors
- AuthContext com estado global
- Components: Button, Input, Spinner, Toast
- Pages: LoginPage, TwoFactorPage, RecoveryPage, ResetPage, DashboardPage
- Setup 2FA com QR Code
- Dockerfile multi-stage
- **dist/ commitado no repositório (precisa rebuild se mudar VITE_API_TARGET)**

### ✅ Docker Compose - CONFIGURADO
- dois serviços: api e frontend
- Network dokploy-network configurada
- Labels de health check
- Pronto para deploy no Dokploy

---

## Deploy no Dokploy - COMPLETO

### ✅ Conferir após deploy
1. API: https://api.agenciadex.com/health → 200 OK
2. Frontend: https://myaccount.agenciadex.com/ → 200 OK

### ⚠️ Frontend - Rebuild Necessário
Se o frontend estiver chamando `localhost:3000` em vez de `https://api.agenciadex.com`:

```bash
cd src/frontend
VITE_API_TARGET=https://api.agenciadex.com npm run build
cd ../..
git add src/frontend/dist/
git commit -m "fix: update API URL in frontend bundle"
git push
```

---

## Backlog - Melhorias Futuras

### ⚠️ Alto Prioridade

#### 1. Rate Limiting - Lockout 15min após 5 falhas
**Descrição:** Implementar bloqueio por 15 minutos após 5 tentativas incorretas no verify-2fa

**Status:** Não implementado

**Arquivos a modificar:**
- `src/middleware/rate_limit.rs` - adicionar novo limiter
- `src/routes/auth.rs` - adicionar lógica de lockout

---

### 📊 Médio Prioridade

#### 2. Métricas Prometheus Custom
**Descrição:** Instrumentar métricas custom:
- `auth_login_total`
- `auth_login_failed_total`
- `auth_2fa_attempts_total`
- `auth_refresh_latency_ms`
- `auth_login_latency_ms`

**Status:** Parcial - exportador existe mas métricas não instrumentadas

**Arquivos a modificar:**
- `src/services/auth.rs` - incrementar contadores e medir latência

---

#### 3. Logging Aprimorado
**Descrição:** Adicionar request_id (UUIDv7), IP, user-agent aos logs

**Status:** Parcial - logs existem mas sem request_id

**Arquivos a criar/modificar:**
- `src/middleware/` - criar RequestIdMiddleware
- `src/routes/auth.rs` - incluir request_id no contexto
- `src/services/auth.rs` - adicionar campos contextuais nos logs

---

#### 4. Otimização Docker Cache
**Descrição:** Criar `src/lib.rs` para separar dependências do código

**Benefício:** Build mais rápido em produção (deps cached)

**Arquivos a criar/modificar:**
- `src/lib.rs` - exportar módulos principais
- `src/main.rs` - adaptar para usar lib

---

### 📊 Baixa Prioridade

#### 5. OpenTelemetry Tracing Completo
**Descrição:** Spans customizados para login, 2fa, refresh, logout

**Crates necessários:**
- opentelemetry
- opentelemetry-otlp
- tracing-opentelemetry

**Status:** Não implementado - infraestrutura não disponível

---

## Notas - Rate Limiting

O rate limiting atual usa `tower-governor` com `SmartIpKeyExtractor`:

| Endpoint | Limite |
|----------|--------|
| `/auth/login` | 1 req/s, burst 5 |
| `/auth/verify-2fa` | 1 req/s, burst 5 |
| `/auth/password/forgot` | 1 req/s, burst 3 |
| Demais endpoints | 10 req/s, burst 50 |

**Pendência:** "bloqueio por 15 minutos após 5 tentativas incorretas" não implementado.

---

## Ordem de Implementação Recomendada

1. **Deploy (agora):**
   - ✅ Deploy com Docker Compose no Dokploy
   - ✅ Verificar funcionamento em produção
   - ⚠️ Rebuild frontend se necessário

2. **Próximas Melhorias:**
   - Rate limiting lockout 15min
   - Métricas Prometheus custom
   - Logging aprimorado

3. **Futuro:**
   - OpenTelemetry tracing
   - Otimização Docker cache