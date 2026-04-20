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
```

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

---

## Estrutura do Projeto

```
dex-account/
├── Cargo.toml
├── Dockerfile                    # Backend Rust multi-stage
├── docker-compose.yml           # Dokploy Docker Compose
├── migrations/
│   └── 20240101000000_initial_schema.sql
├── src/
│   ├── main.rs
│   ├── bin/dex-account-recovery.rs
│   ├── db/mod.rs
│   ├── error/mod.rs
│   ├── middleware/
│   │   ├── auth.rs
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
│   │   ├── pages/
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── Dockerfile
│   ├── dist/                    # Build pré-compilado
│   ├── package.json
│   └── vite.config.ts
├── Docs/
│   ├── API.md
│   ├── DOKPLOY.md
│   └── TODO.md
├── .env.example
└── .gitignore
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

---

## Tarefas Pendentes

Veja [Docs/TODO.md](Docs/TODO.md) para lista completa de tarefas.
