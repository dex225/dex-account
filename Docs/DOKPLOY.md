# Configuração no Dokploy

## 1. Pré-requisitos

- Servidor Dokploy instalado e configurado
- Repositório Git conectado ao Dokploy (GitHub, GitLab, etc.)

## 2. Criar o Banco de Dados PostgreSQL

1. Vá em **Databases** > **Create Database**
2. Selecione **PostgreSQL**
3. Configure:
   - **Name:** `dex_account`
   - **Default Database:** `dex_account`
4. Após criar, vá em **Connection** para obter a `DATABASE_URL`

## 3. Criar o Projeto

1. Vá em **Projects** > **Create Project**
2. **Name:** `dex-account`

## 4. Criar a Aplicação (2 containers com docker-compose)

1. Dentro do projeto, clique em **Create Application**
2. Configure:

### General

- **Name:** `dex-account`
- **Build Type:** `docker-compose` (para múltiplos containers)
- **Repository:** `https://github.com/dex225/dex-account`
- **Branch:** `main`

### Docker Compose

Crie o arquivo `docker-compose.yml` na raiz do projeto:

```yaml
services:
  api:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - DEX_JWT_SECRET=${DEX_JWT_SECRET}
      - DEX_EMERGENCY_API_KEY=${DEX_EMERGENCY_API_KEY}
      - DEX_ALLOWED_ORIGINS=${DEX_ALLOWED_ORIGINS}
      - DEX_AUTO_MIGRATE=false
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
    restart: unless-stopped

  frontend:
    build:
      context: .
      dockerfile: src/frontend/Dockerfile
    ports:
      - "80:80"
    environment:
      - VITE_API_TARGET=https://api.agenciadex.com
    depends_on:
      - api
    restart: unless-stopped
```

### Configurar Domínios

1. Vá em **Domains** na aplicação
2. Configure os domínios para cada serviço:

| Serviço | Domínio | Porta |
|---------|---------|-------|
| `frontend` | `myaccount.agenciadex.com` | 80 |
| `api` | `api.agenciadex.com` | 3000 |

### Ordem de Deploy

1. Deploy primeiro o container `api` (ou toda a aplicação de uma vez)
2. O Traefik do Dokploy roteará cada domínio para o container correto

## 5. Variáveis de Ambiente

No Dokploy, variáveis podem ser definidas em três níveis:

### Variáveis de Projeto (compartilhadas)

No projeto, defina:

```
DATABASE_URL=${{pg_dex_account.CONNECTION_URI}}
```

### Variáveis da Aplicação (docker-compose)

Na aplicação, defina:

```
DEX_JWT_SECRET=sua-chave-secreta-minimo-32-caracteres
DEX_EMERGENCY_API_KEY=sua-chave-de-emergencia
DEX_ALLOWED_ORIGINS=https://myaccount.agenciadex.com
```

### Ordem de deploy

1. Deploy a aplicação docker-compose (ambos containers são criados juntos)

### Referenciando variáveis

O Dokploy permite referenciar variáveis de outros níveis:

```env
DATABASE_URL=${{project.DATABASE_URL}}
```

## 6. Configurar Domínio

1. Vá em **Domains** na aplicação
2. Clique em **Create Domain**
3. Configure:
   - **Domain:** `auth.seudominio.com`
   - **HTTPS:** sim (Let's Encrypt automático)

Ou use domínio gerado: clique no ícone de dados para gerar um domínio `.traefik.me`.

## 7. Deploy - Produção Recomendada

Para produção, é recomendado usar CI/CD com GitHub Actions. Isso separa o build da execução de migrations.

### 7.1 Configurar GitHub Actions

Crie o arquivo `.github/workflows/deploy.yml`:

```yaml
name: Build, Migrate and Deploy

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile
          push: true
          tags: ghcr.io/${{ github.repository }}/dex-account:latest
          platforms: linux/amd64

  migrate:
    needs: build
    runs-on: ubuntu-latest
    container: postgres
    steps:
      - name: Run migrations
        run: |
          PGPASSWORD=${{ secrets.DB_PASSWORD }} psql \
            -h ${{ secrets.DB_HOST }} \
            -U ${{ secrets.DB_USER }} \
            -d dex_account \
            -f migrations/20240101000000_initial_schema.sql

  deploy:
    needs: migrate
    runs-on: ubuntu-latest
    steps:
      - name: Trigger Dokploy Deploy
        uses: dokploy/dokploy-action@v1
        with:
          api-key: ${{ secrets.DOKPLOY_API_KEY }}
          application-id: ${{ secrets.DOKPLOY_APP_ID }}
```

### 7.2 Secrets no GitHub

Configure os seguintes secrets em **Settings** > **Secrets and variables** > **Actions**:

| Secret | Descrição |
|--------|----------|
| `DB_HOST` | Host do banco PostgreSQL |
| `DB_USER` | Usuário do banco |
| `DB_PASSWORD` | Senha do banco |
| `DOKPLOY_API_KEY` | API Key do Dokploy |
| `DOKPLOY_APP_ID` | ID da aplicação no Dokploy |

### 7.3 Alternativa: Build e Deploy pelo Dokploy

Se preferir fazer o build pelo Dokploy:

1. **Primeiro deploy (com migrations):**

   - Adicione a variável: `DEX_AUTO_MIGRATE=true`
   - Faça o deploy pelo Dokploy
   - As migrations rodarão automaticamente

2. **Deploys subsequentes (sem migrations):**

   - Remova ou defina: `DEX_AUTO_MIGRATE=false`
   - Deploy pelo Dokploy

## 8. Configurar Health Check

Para rollbacks automáticos em caso de falha:

1. Vá em **Advanced** > **Swarm Settings**
2. Configure **Health Check**:

```json
{
  "Test": ["CMD", "curl", "-f", "http://localhost:3000/health"],
  "Interval": 30000000000,
  "Timeout": 10000000000,
  "StartPeriod": 30000000000,
  "Retries": 3
}
```

3. Configure **Update Config**:

```json
{
  "Parallelism": 1,
  "Delay": 10000000000,
  "FailureAction": "rollback",
  "Order": "start-first"
}
```

## 9. Variáveis de Ambiente Resumidas

| Variável | Obrigatório | Descrição |
|----------|-------------|-----------|
| `DATABASE_URL` | Sim | Connection string PostgreSQL |
| `DEX_JWT_SECRET` | Sim | Segredo JWT (mín. 32 chars) |
| `DEX_EMERGENCY_API_KEY` | Sim | Chave de emergência |
| `DEX_ALLOWED_ORIGINS` | Sim | URLs CORS (separadas por vírgula) |
| `DEX_AUTO_MIGRATE` | Não | Executa migrations automaticamente (padrão: false) |
| `DEX_CLEANUP_INTERVAL_HOURS` | Não | Intervalo cleanup (padrão: 1) |

## 10. Estrutura de Variáveis no Dokploy

```
Projeto (shared)
└── DATABASE_URL=${{pg_dex_account.CONNECTION_URI}}

Aplicação
├── DEX_JWT_SECRET=minha-chave
├── DEX_EMERGENCY_API_KEY=chave-emergencia
└── DEX_ALLOWED_ORIGINS=https://app.exemplo.com
```

## 11. Troubleshooting

### Container não inicia

```bash
# Ver logs em tempo real
dokploy logs -f dex-account

# Verificar variáveis
dokploy inspect dex-account
```

### Erro de conexão banco

1. Verificar se banco e app estão na mesma rede
2. Confirmar `DATABASE_URL` correto
3. Testar do container: `docker exec dex-account curl localhost:5432`

### CORS errors

Garantir que `DEX_ALLOWED_ORIGINS` contém exatamente as URLs do frontend, sem espaços.

### Migrations não rodam

1. Verificar se `DEX_AUTO_MIGRATE=true`
2. Verificar logs de migrations
3. Executar manualmente se necessário via Exec do container

## 12. Segurança em Produção

### Variáveis Sensíveis

- Nunca commit variáveis com senhas/secrets
- Usar GitHub Secrets ou variáveis do Dokploy
- Rotacionar `DEX_EMERGENCY_API_KEY` periodicamente

### Network

- Banco deve estar em rede Docker isolada
- Usar `localhost` ou nome do serviço Docker para conexão interna
- Nunca expor porta do banco para internet

### Monitoramento

- Configurar alerts para `/health` e `/ready`
- Monitorar logs de erros
- Configurar backups automáticos do banco via Dokploy
