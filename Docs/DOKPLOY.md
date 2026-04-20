# Configuração no Dokploy

## 1. Preparar o Projeto no GitHub

O projeto já está configurado em: `https://github.com/dex225/dex-account`

Certifique-se de que o repositório está conectado ao Dokploy.

## 2. Criar o Banco de Dados

1. No painel Dokploy, vá em **Databases**
2. Clique em **Create Database**
3. Selecione **PostgreSQL**
4. Configure:
   - **Name:** `dex_account`
   - **User:** `dex_account`
   - **Password:** (generate secure password)
5. Anote a **Connection URL** (formato: `postgres://user:pass@host:5432/dex_account`)

## 3. Criar o Projeto no Dokploy

1. Vá em **Projects** > **Create Project**
2. **Name:** `dex-account`
3. **Type:** Private
4. **Git Repository:** `https://github.com/dex225/dex-account`

## 4. Criar o Servidor (Server)

1. Vá em **Servers** > **Create Server**
2. Configure o servidor Docker onde o container será deployed

## 5. Criar o App (Docker)

1. Vá em **Projects** > **dex-account** > **Create App**
2. Configure:

### General

- **Name:** `dex-account`
- **App Type:** `Docker`
- **Server:** (selecione o servidor criado)
- **Port:** `3000`

### Build

- **Dockerfile Location:** `Dockerfile`
- **Build Method:** `nixpacks` ou `dockerfile`

### Environment Variables

Adicione todas as variáveis obrigatórias:

```env
DATABASE_URL=postgres://dex_account:SUA_SENHA@host_dokploy:5432/dex_account
DEX_JWT_SECRET=gerar-uma-chave-secreta-com-minimo-32-caracteres
DEX_EMERGENCY_API_KEY=gerar-uma-chave-aleatoria-segura
DEX_ALLOWED_ORIGINS=https://myaccount.seudominio.com,https://app.seudominio.com
```

### Persistent Storage (opcional)

Se desejar persistência de dados:
- **Volume:** `/var/lib/postgresql/data`
- **Mount Path:** `/data`

## 6. Configurar o Dockerfile

O Dokploy pode usar o Dockerfile existente. Certifique-se de que ele está na raiz:

```dockerfile
# build stage
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists
COPY --from=builder /app/target/release/dex-account /usr/local/bin/
EXPOSE 3000
CMD ["dex-account"]
```

Ou use multi-stage build com `scratch` para imagem mínima:

```dockerfile
# stage 1
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# stage 2
FROM alpine:edge
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/dex-account /usr/local/bin/
EXPOSE 3000
CMD ["dex-account"]
```

## 7. Configurar o Domínio (opcional)

1. Vá em **App** > **dex-account** > **Domains**
2. Clique em **Add Domain**
3. Configure:
   - **Domain:** `auth.seudominio.com`
   - **HTTPS:** sim (Let's Encrypt)
   - **WWW Redirect:** não

## 8. Migrar o Banco

1. Vá em **App** > **dex-account** > **Containers**
2. Clique no container e depois em **Exec**
3. Execute o comando de migration:

```bash
# Se usar sqlx-cli
sqlx migrate run

# Ou via psql diretamente
psql $DATABASE_URL -f migrations/001_initial_schema.sql
```

## 9. Verificar Health Checks

1. Acesse `/health` e `/ready` para confirmar que o serviço está rodando
2. Monitore os logs no painel do Dokploy

## Variáveis de Ambiente Resumidas

| Variável | Descrição |
|----------|----------|
| `DATABASE_URL` | PostgreSQL connection string |
| `DEX_JWT_SECRET` | Segredo JWT (mín. 32 chars) |
| `DEX_EMERGENCY_API_KEY` | Chave de recuperação de emergência |
| `DEX_ALLOWED_ORIGINS` | URLs CORS (separadas por vírgula) |
| `DEX_CLEANUP_INTERVAL_HOURS` | Intervalo cleanup (padrão: 1) |

## Troubleshooting

### Container não inicia

```bash
# Ver logs
docker logs dex-account

# Verificar variáveis
docker exec dex-account env
```

### Erro de conexão com banco

1. Verificar se o banco está na mesma rede Docker
2. Confirmar `DATABASE_URL` correto
3. Testar conexão: `docker exec dex-account psql $DATABASE_URL -c "SELECT 1"`

### CORS errors

Confirmar que `DEX_ALLOWED_ORIGINS` contém exatamente as URLs do frontend, sem espaços.
