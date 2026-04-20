# DEX Account

Serviço de autenticação (IAM) para o ecossistema da Digital Expansion.

## Stack

- **Linguagem:** Rust
- **Framework:** Axum
- **Banco de Dados:** PostgreSQL
- **ORM:** SQLx

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
docker build --target production -t dex-account .
```

### Build Multi-stage (scratch)

```bash
docker build -t dex-account .
```

## API

Veja [Docs/API.md](Docs/API.md) para documentação completa dos endpoints.
