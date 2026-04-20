# Tarefas Pendentes - DEX Account

## Backend (Rust/Axum)

### ✅ Concluído
- Login/logout com JWT + RTR (Refresh Token Rotation)
- 2FA com TOTP
- Recuperação de senha
- Recuperação de emergência (endpoint + CLI)
- Health checks (/health, /ready)
- Rate limiting (tower-governor) - 4 limiters configurados
- Prometheus metrics exporter (porta 3001)
- Migrations automáticas (DEX_AUTO_MIGRATE)
- Docker multi-stage build
- Cleanup automático de tokens expirados
- Tracing/logging básico (tracing crate)

### ⚠️ Concluído (Implementação Parcial)

#### Métricas Prometheus Custom
- Exportador Prometheus configurado na porta 3001 ✅
- Métricas custom `login_total`, `login_failed_total`, `2fa_attempts_total`, `login_latency_ms` **não instrumentadas** ❌
- Arquivos: `src/services/metrics.rs` existe mas precisa de instrumentação completa em `auth.rs`

#### Logging Aprimorado
- Logging básico com tracing ✅
- `request_id` (UUIDv7), IP, user-agent, user_id (hashed) **não implementados** ❌
- Middleware `RequestIdMiddleware` não existe

#### Rate Limiting
- Tower-governor com 4 limiters ✅
- Bloqueio por 15 minutos após 5 tentativas incorretas no verify-2fa **não implementado** ❌

### ⏳ A Fazer (Pré-Produção)

#### 1. OpenTelemetry Tracing Completo
**Referência SDD:** Seção 13 - Observabilidade

**Descrição:**
- Implementar spans customizados para: login, 2fa, refresh, logout, emergency-recover
- Attributes: user_id (hashed), IP, user-agent, status_code
- Exportação OTLP para coletor (Jaeger, Tempo, Grafana)

**Crates necessários:**
```toml
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"
tracing-opentelemetry = "0.22"
```

**Variáveis de ambiente necessárias:**
- `OTEL_EXPORTER_OTLP_ENDPOINT` - URL do coletor OTLP
- `OTEL_SERVICE_NAME` - nome do serviço (ex: "dex-account")

**Arquivos a modificar:**
- `src/main.rs` - inicialização do OTLP tracer
- `src/services/auth.rs` - adicionar spans nas operações
- `src/middleware/` - criar middleware de tracing

**Status:** Não implementado - infraestrutura de coleta não disponível

---

#### 2. Métricas Prometheus Custom
**Referência SDD:** Seção 13 - Observabilidade

**Descrição:**
Implementar as métricas definidas no SDD:

| Métrica | Tipo | Descrição |
|---------|------|-----------|
| `auth_login_total` | Counter | Total de logins bem-sucedidos |
| `auth_login_failed_total` | Counter | Total de logins falhados |
| `auth_2fa_attempts_total` | Counter | Total de tentativas 2FA |
| `auth_refresh_latency_ms` | Histogram | Latência do refresh token |
| `auth_login_latency_ms` | Histogram | Latência do login |

**Arquivos a modificar:**
- `src/services/auth.rs` - incrementar contadores e medir latência

**Status:** Parcial - exportador existe mas métricas custom não foram instrumentadas

---

#### 3. Logging Aprimorado
**Referência SDD:** Seção 14 - Logging

**Descrição:**
Melhorar o logging para incluir:
- `request_id` (UUIDv7) gerado por middleware - **FALTA IMPLEMENTAR**
- `user_id` (hashed paraanonimizar) quando disponível
- `ip` do cliente
- `user_agent`

**Campos proibidos de logar (SDD):**
- Senhas
- Tokens / refresh_tokens
- totp_secret
- api_keys

**Arquivos a modificar:**
- `src/middleware/` - criar RequestIdMiddleware
- `src/routes/auth.rs` - incluir request_id no contexto
- `src/services/auth.rs` - adicionar campos contextuais nos logs

**Status:** Parcial - logs existem mas sem request_id

---

## Frontend (React)

**Referência SDD:** Seção 16 - Frontend (DEX Auth UI)

**Status:** Não iniciado

### Tarefas:
1. Decidir entre Tailwind CSS ou CSS Modules
2. Criar estrutura `/src/frontend/` com Vite + React + TypeScript
3. Implementar páginas:
   - `/` - Login
   - `/2fa` - Verificação 2FA
   - `/recovery` - Recuperação de senha
   - `/dashboard` - Painel do usuário
4. Criar Dockerfile (multi-stage com node:alpine)
5. Configurar CI/CD (GitHub Actions)
6. Criar app `dex-auth-ui` no Dokploy

---

## Infraestrutura

### Pré-Produção
- [ ] Configurar coletor OTLP (Jaeger ou Grafana Tempo)
- [ ] Configurar Grafana Dashboard para métricas Prometheus
- [ ] Testar procedure de backup/restore
- [ ] Configurar alertas (rate limit, errors, latency)

### Produção
- [ ] Definir domínio do frontend (ex: `https://auth.dex.com.br`)
- [ ] Atualizar CORS no backend com domínio do frontend
- [ ] Configurar SSL/TLS (Dokploy/Traefik com Let's Encrypt)
- [ ] Definir estratégia de rollback

---

## Notas

### Limitação de Taxa
O rate limiting atual usa `tower-governor` com as seguintes configurações:
- Login: 1 req/sec, burst 5
- Verify 2FA: 1 req/sec, burst 5
- Password Forgot: 1 req/sec, burst 3
- General: 10 req/sec, burst 50

**Pendência SDD:** "bloqueio por 15 minutos após 5 tentativas incorretas" no verify-2fa não implementado.

---

## Ordem de Implementação Recomendada

1. **Agora (Desenvolvimento):**
   - ✅ Backend funcional
   - ⏳ Métricas Prometheus custom (`login_total`, `login_latency_ms`, `refresh_latency_ms`) - 2h
   - ⏳ Logging com request_id (UUIDv7), IP, user-agent - 1h
   - ⏳ Rate limiting: lockout 15min após 5 falhas no verify-2fa - 1h

2. **Pré-Produção:**
   - OpenTelemetry tracing
   - Infraestrutura de observabilidade

3. **Produção:**
   - CI/CD completo
   - SSL/Domínios
   - Alertas
