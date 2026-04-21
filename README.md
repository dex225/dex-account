# DEX Account

Serviço de autenticação (IAM) para o ecossistema da Digital Expansion.

## Stack

- **Backend:** Rust + Axum + PostgreSQL + SQLx
- **Frontend:** React 18 + TypeScript + Vite + Tailwind CSS
- **Infraestrutura:** Docker Compose + Dokploy
- **Cache de métricas:** Prometheus

## Requisitos

- Rust 1.75+
- PostgreSQL 14+
- Node.js 18+ (para desenvolvimento frontend)
- Docker (para desenvolvimento)

## Deploy com Dokploy

### 1. Configurar Banco de Dados

1. No Dokploy, crie um banco PostgreSQL
2. Anote a `DATABASE_URL` fornecida

### 2. Criar Serviço Docker Compose

1. No projeto Dokploy, clique em **Create Service**
2. Selecione **Compose Type: Docker Compose**
3. Configure:
   - **Compose Path:** `./docker-compose.yml`
   - **Repository:** seu repositório Git
   - **Branch:** `main`

### 3. Variáveis de Ambiente

```env
DATABASE_URL=${{pg_dex_account.CONNECTION_URI}}
DEX_JWT_SECRET=sua-chave-secreta-minimo-32-caracteres
DEX_EMERGENCY_API_KEY=sua-chave-de-emergencia
DEX_ALLOWED_ORIGINS=https://myaccount.agenciadex.com
DEX_AUTO_MIGRATE=false
DEX_SETUP_TOKEN=token-para-criar-admin-inicial
```

**Importante:** Após criar o primeiro admin via `/auth/setup`, remova ou altere o `DEX_SETUP_TOKEN`.

### 4. Configurar Domínios

| Serviço | Domínio | Porta |
|---------|---------|-------|
| `api` | `api.agenciadex.com` | 3000 |
| `frontend` | `myaccount.agenciadex.com` | 80 |

### 5. Deploy!

Consulte [Docs/DOKPLOY.md](Docs/DOKPLOY.md) para instruções completas.

---

## Desenvolvimento

### Preparar Ambiente

```bash
# Clonar repositório
git clone https://github.com/dex225/dex-account.git
cd dex-account

# Configurar banco local (opcional)
createdb dex_account
cp .env.example .env
# Editar .env com credenciais locais
```

### Backend

```bash
# Aplicar migrations
sqlx migrate run

# Executar
cargo run
```

### Frontend (desenvolvimento)

```bash
cd src/frontend
npm install
npm run dev
```

---

## API

Consulte [Docs/API.md](Docs/API.md) para documentação completa dos endpoints.

## Funcionalidades

- [x] Login/logout com JWT + Refresh Token Rotation (RTR)
- [x] Autenticação em dois fatores (2FA) com TOTP
- [x] Recuperação de senha
- [x] Recuperação de emergência
- [x] Rate limiting por IP (funciona com Traefik/Dokploy)
- [x] Métricas Prometheus (porta 3001)
- [x] Health checks (/health, /ready)
- [x] Cleanup automático de tokens expirados
- [x] Migrations automáticas
- [x] Docker Compose configurado para Dokploy
- [x] Middleware de autenticação para rotas protegidas
- [x] Setup inicial via `/auth/setup` para criar primeiro admin

---

## Estrutura do Projeto

```
dex-account/
├── Cargo.toml
├── Dockerfile                    # Backend Rust multi-stage
├── Dockerfile.frontend          # Frontend build stage
├── docker-compose.yml           # Dokploy Docker Compose
├── .env.example
├── .env.production              # Vars build-time do frontend
├── .gitignore
├── .dockerignore
├── migrations/
│   └── 20240101000000_initial_schema.sql
├── src/                         # Backend Rust
│   ├── main.rs
│   ├── db/mod.rs
│   ├── error/mod.rs
│   ├── middleware/
│   │   ├── auth.rs             # Middleware de autenticação JWT
│   │   ├── mod.rs
│   │   └── rate_limit.rs
│   ├── models/mod.rs
│   ├── routes/
│   │   ├── auth.rs
│   │   └── mod.rs
│   └── services/
│       ├── auth.rs
│       ├── crypto.rs
│       ├── metrics.rs
│       └── mod.rs
├── src/frontend/                # Frontend React
│   ├── src/
│   │   ├── components/
│   │   ├── context/
│   │   ├── lib/
│   │   │   ├── api.ts          # Cliente API com interceptors
│   │   │   └── constants.ts
│   │   ├── pages/
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── Dockerfile
│   ├── nginx.conf              # Config nginx para SPA
│   ├── dist/                    # Build pré-compilado
│   ├── package.json
│   └── vite.config.ts
└── Docs/
    ├── API.md                   # Referência completa da API
    ├── DOKPLOY.md               # Guia de deploy no Dokploy
    └── TODO.md
```

---

## Rate Limiting

O serviço implementa rate limiting por IP usando `tower-governor` com `SmartIpKeyExtractor`:

| Endpoint | Limite |
|----------|--------|
| `/auth/login` | 1 req/s, burst 5 |
| `/auth/verify-2fa` | 1 req/s, burst 5 |
| `/auth/password/forgot` | 1 req/s, burst 3 |
| Demais endpoints | 10 req/s, burst 50 |

---

## Monitoramento

- **Health:** `GET /health` - Liveness probe
- **Ready:** `GET /ready` - Readiness probe (verifica DB)
- **Metrics:** `GET :3001/metrics` - Métricas Prometheus

---

## Segurança - Produção

1. **Migrations:** Execute via CI/CD ou na primeira vez com `DEX_AUTO_MIGRATE=true`
2. **Secrets:** Use variáveis de ambiente do Dokploy
3. **CORS:** Configure apenas origens confiáveis
4. **JWT Secret:** Use chave de no mínimo 32 caracteres
5. **Emergency Key:** Guarde em cofre de senhas

---

## Troubleshooting

### Erro CORS

Verifique se `DEX_ALLOWED_ORIGINS` contém exatamente as URLs do frontend, sem espaços.

### Erro de conexão com banco

1. Verificar se banco está acessível
2. Confirmar `DATABASE_URL` correto
3. Verificar logs do container

### Frontend 502 Bad Gateway

1. Verificar se o container `frontend` está rodando
2. Verificar logs do container frontend
3. Confirmar que o domínio está configurado para porta 80

### Frontend chama localhost:3000 em vez da API correta

O frontend é buildado com `VITE_API_TARGET` hardcoded no bundle. Se a URL da API estiver errada:
1. Edite `src/frontend/.env.production` com a URL correta
2. Rebuild: `cd src/frontend && npm run build`
3. Commit e push do novo `dist/`

---

## Criar Primeiro Admin

Após o primeiro deploy, use o endpoint `/auth/setup`:

```bash
curl -X POST https://api.agenciadex.com/api/v1/auth/setup \
  -H "Content-Type: application/json" \
  -d '{
    "token": "seu-DEX_SETUP_TOKEN",
    "email": "admin@agenciadex.com",
    "password": "SuaSenhaForte123"
  }'
```

Consulte [Docs/DOKPLOY.md](Docs/DOKPLOY.md) para instruções completas.

---

## Tarefas Pendentes

Veja [Docs/TODO.md](Docs/TODO.md) para lista completa de tarefas.
