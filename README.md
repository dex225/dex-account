# DEX Account

Serviço de autenticação (IAM) para o ecossistema da Digital Expansion.

## Stack

- **Linguagem:** Rust
- **Framework:** Axum
- **Banco de Dados:** PostgreSQL
- **ORM:** SQLx
- **Cache de métricas:** Prometheus

## Requisitos

- Rust 1.75+
- PostgreSQL 14+
- Docker (para desenvolvimento)

## Configuração

Copie `.env.example` para `.env` e configure as variáveis:

```bash
cp .env.example .env
```

### Variáveis de Ambiente

| Variável | Obrigatório | Descrição |
|----------|-------------|-----------|
| `DATABASE_URL` | Sim | String de conexão PostgreSQL |
| `DEX_JWT_SECRET` | Sim | Segredo para assinar JWTs (mín. 32 caracteres) |
| `DEX_EMERGENCY_API_KEY` | Sim | Chave para recuperação de emergência |
| `DEX_ALLOWED_ORIGINS` | Sim | URLs permitidas para CORS (separadas por vírgula) |
| `DEX_AUTO_MIGRATE` | Não | Executa migrations automaticamente (padrão: false) |
| `DEX_CLEANUP_INTERVAL_HOURS` | Não | Intervalo para cleanup de tokens expirados (padrão: 1) |

## Desenvolvimento

```bash
# Preparar banco de dados
createdb dex_account

# Aplicar migrations
sqlx migrate run

# Executar
cargo run
```

## Produção

### Build Docker

```bash
docker build -t dex-account .
```

### Deploy com Dokploy

Consulte [Docs/DOKPLOY.md](Docs/DOKPLOY.md) para instruções completas de deployment.

### Deploy com CI/CD (GitHub Actions)

Consulte [Docs/DOKPLOY.md](Docs/DOKPLOY.md#71-configurar-github-actions) para configuração de CI/CD.

## API

Veja [Docs/API.md](Docs/API.md) para documentação completa dos endpoints.

## Funcionalidades

- [x] Login/logout com JWT + Refresh Token Rotation (RTR)
- [x] Autenticação em dois fatores (2FA) com TOTP
- [x] Recuperação de senha
- [x] Recuperação de emergência
- [x] Rate limiting por IP
- [x] Métricas Prometheus (porta 3001)
- [x] Health checks (/health, /ready)
- [x] Cleanup automático de tokens expirados
- [x] Migrations automáticas

## Estrutura do Projeto

```
dex-account/
├── Cargo.toml
├── Dockerfile
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
├── Docs/
│   ├── API.md
│   ├── DOKPLOY.md
│   └── TODO.md
├── .env.example
└── .gitignore
```

## Segurança

### Produção

1. **Migrations:** Execute via CI/CD (não use `DEX_AUTO_MIGRATE=true` em produção)
2. **Secrets:** Use variáveis de ambiente ou GitHub Secrets
3. **CORS:** Configure apenas origens confiáveis
4. **JWT Secret:** Use chave de no mínimo 32 caracteres
5. **Emergency Key:** Guarde em cofre de senhas

### Ambiente Development

Para desenvolvimento local, você pode usar `DEX_AUTO_MIGRATE=true` para rodar migrations automaticamente.

## Rate Limiting

O serviço implementa rate limiting por IP usando `tower-governor`:

| Endpoint | Limite |
|----------|--------|
| `/auth/login` | 1 req/s, burst 5 |
| `/auth/verify-2fa` | 1 req/s, burst 5 |
| `/auth/password/forgot` | 1 req/s, burst 3 |
| Demais endpoints | 10 req/s, burst 50 |

## Monitoring

- **Health:** `GET /health` - Liveness probe
- **Ready:** `GET /ready` - Readiness probe (verifica DB)
- **Metrics:** `GET :3001/metrics` - Métricas Prometheus

## Troubleshooting

### Erro CORS

Verifique se `DEX_ALLOWED_ORIGINS` contém exatamente as URLs do frontend, sem espaços.

### Erro de conexão com banco

1. Verificar se o banco está rodando
2. Confirmar `DATABASE_URL` correto
3. Testar conexão: `psql $DATABASE_URL -c "SELECT 1"`

### Container não inicia

```bash
docker logs dex-account
docker exec dex-account env
```

## Tarefas Pendentes

Veja [Docs/TODO.md](Docs/TODO.md) para lista de tarefas pendientes (pré-produção e produção).