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
- Middleware de autenticação para rotas protegidas
- Setup inicial via `/auth/setup` para criar primeiro admin
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

#### 1. Persistência de Sessão (Silent Refresh)
**Descrição:** Ao abrir o app, dispara request silencioso para `/auth/refresh`. Se cookie de refresh for válido, restaura sessão automaticamente. O access token fica em memória (RAM), não localStorage - mantendo a segurança do HttpOnly cookie.

**Status:** ✅ Implementado

**Arquivos modificados:**
- `src/frontend/src/context/AuthContext.tsx` - useEffect no mount para silent refresh
- `src/routes/auth.rs` - login/verify-2fa/refresh setam `Set-Cookie: refresh_token=...; HttpOnly; SameSite=Strict; Secure; Path=/; Max-Age=1296000`

**Fluxo:**
1. App abre → `isLoading: true`
2. `refreshAccessToken()` chamado automaticamente
3. Cookie HttpOnly enviado com credentials
4. Se válido → access token em memória, usuário logado
5. Se inválido → `isLoading: false`, mostra login

**Segurança:** Refresh token protegido por cookie HttpOnly + RTR. Access token nunca sai da memória RAM.

---

#### 2. IP Lockout após 5 tentativas falhadas
**Descrição:** Implementar bloqueio por 15 minutos após 5 tentativas incorretas no login e verify-2fa. Após bloqueio, retorna erro 429 "Too many failed attempts. Account locked for X minutes".

**Status:** ✅ Implementado

**Arquivos modificados:**
- `src/middleware/ip_lockout.rs` - tracking de tentativas por IP (DashMap)
- `src/middleware/client_ip.rs` - extração de IP dos headers (X-Forwarded-For, X-Real-IP)
- `src/error/mod.rs` - nova variante `IpLocked(u64)`
- `src/routes/auth.rs` - lockout check em login e verify_2fa

**Configuração:**
- Max tentativas: 5
- Lockout duration: 15 minutos

---

### 📊 Médio Prioridade

#### 4. Métricas Prometheus Custom
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

#### 5. Logging Aprimorado
**Descrição:** Adicionar request_id (UUIDv7), IP, user-agent aos logs

**Status:** Parcial - logs existem mas sem request_id

**Arquivos a criar/modificar:**
- `src/middleware/` - criar RequestIdMiddleware
- `src/routes/auth.rs` - incluir request_id no contexto
- `src/services/auth.rs` - adicionar campos contextuais nos logs

---

#### 6. Otimização Docker Cache
**Descrição:** Criar `src/lib.rs` para separar dependências do código

**Benefício:** Build mais rápido em produção (deps cached)

**Arquivos a criar/modificar:**
- `src/lib.rs` - exportar módulos principais
- `src/main.rs` - adaptar para usar lib

---

### 📊 Baixa Prioridade

#### 7. OpenTelemetry Tracing Completo
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

**IP Lockout:** Após 5 tentativas incorretas em `/auth/login` ou `/auth/verify-2fa`, o IP é bloqueado por 15 minutos (retorna 429).

---

## Problemas Conhecidos

### Erro no console ao abrir /2fa (Bootstrap Autofill)

**Descrição:** Ao abrir a página `/2fa`, aparece erro no console:
```
bootstrap-autofill-overlay.js:1269 Uncaught (in promise) NotFoundError: Failed to execute 'insertBefore' on 'Node': The node before which the new node is to be inserted is not a child of this node.
```

**Causa:** Erro interno do módulo de preenchimento automático do browser (Bitwarden/1Password). O código da aplicação não é a causa direta - é uma interferência da extensão do gerenciador de senhas ao manipular o DOM.

**Status:** Bug externo - extensão do browser

**Solução:**
1. Testar desabilitando extensões de preenchimento automático (Bitwarden, 1Password, etc)
2. Se persistir, verificar se há alguma interferência com o QR code ou input de 2FA
3. Este erro não afeta a funcionalidade do 2FA

---

### Bug no fluxo 2FA - botão começa a girar antes do usuário digitar código

**Descrição:** Ao fazer login e ser redirecionado para `/2fa`, o botão "Verificar" já entra no estado `isLoading` (girando) antes do usuário digitar o código ou clicar no botão. Quando o usuário digita o código, nada acontece.

**Causa:** No `AuthContext.tsx`, quando `login()` retorna `challenge_token` (2FA necessário), a função retornava sem resetar `isLoading` para `false`. O `isLoading` ficava `true` permanentemente.

**Status:** ✅ Corrigido

**Correção:**
```javascript
// AuthContext.tsx - login()
if ('access_token' in result) {
  // login sem 2FA
  setState({ ... isLoading: false });
} else {
  // login com 2FA - resetar isLoading também!
  setState((prev) => ({ ...prev, isLoading: false }));
}
```

**Arquivo modificado:** `src/frontend/src/context/AuthContext.tsx`

---

### Timer do 2FA não corresponde ao tempo real do código TOTP

**Descrição:** O timer na página `/2fa` mostra uma contagem decrescente de 5 minutos (hardcoded), mas os códigos TOTP expiram a cada 30 segundos no Google Authenticator. O utilizador não sabe quando o código realmente vai expirar.

**Causa:** O `CHALLENGE_EXPIRY_SECONDS = 300` (5 minutos) está hardcoded em `src/frontend/src/lib/constants.ts`, mas o backend retorna o `expires_in` real no `challenge_token`. Além disso, códigos TOTP têm vida útil de ~30 segundos.

**Status:** Bug - a ser corrigido

**Solução:**
1. Usar o `expires_in` retornado pelo `/auth/login` para calcular o tempo real
2. Idealmente sincronizar com o ciclo de 30 segundos do TOTP

---

## Ordem de Implementação Recomendada

1. **Deploy (agora):**
   - ✅ Deploy com Docker Compose no Dokploy
   - ✅ Verificar funcionamento em produção
   - ✅ Criar admin via `/auth/setup`

2. **Próximas Melhorias:**
   - ✅ Persistência de sessão (Silent Refresh)
   - ✅ Rate limiting lockout 15min
   - Métricas Prometheus custom
   - Logging aprimorado

3. **Futuro:**
   - OpenTelemetry tracing
   - Otimização Docker cache