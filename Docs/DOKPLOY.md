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

## 4. Criar a Aplicação

1. Dentro do projeto, clique em **Create Application**
2. Configure:

### General

- **Name:** `dex-account`
- **Build Type:** `Dockerfile` (recomendado para produção)
- **Repository:** `https://github.com/dex225/dex-account`
- **Branch:** `main`

### Dockerfile

- **Dockerfile Path:** `Dockerfile`
- **Docker Context Path:** `.`

## 5. Variáveis de Ambiente

No Dokploy, variáveis podem ser definidas em três níveis:

### Variáveis de Projeto (compartilhadas)

No projeto, defina:

```
DATABASE_URL=postgres://dex_account:SUA_SENHA@host:5432/dex_account
```

### Variáveis da Aplicação

Na aplicação, defina:

```
DEX_JWT_SECRET=sua-chave-secreta-minimo-32-caracteres
DEX_EMERGENCY_API_KEY=sua-chave-de-emergencia
DEX_ALLOWED_ORIGINS=https://myaccount.seudominio.com,https://app.seudominio.com
```

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

## 7. Build e Deploy

### Opção A: Build no Dokploy (Desenvolvimento)

1. Clique em **Deploy** no painel da aplicação
2. O Dokploy construirá a imagem usando o Dockerfile
3. Aguarde até o deploy completar

### Opção B: Build Externo + Deploy (Produção Recomendada)

Conforme a documentação oficial, para produção é recomendado buildar externamente:

1. **Configure GitHub Actions** para buildar e pushar ao registry:

```yaml
name: Build and Push

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile
          push: true
          tags: seu-dockerhub/dex-account:latest
```

2. **Crie a aplicação no Dokploy** com:
   - **Build Type:** `Docker`
   - **Docker Image:** `seu-dockerhub/dex-account:latest`

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
| `DEX_CLEANUP_INTERVAL_HOURS` | Não | Intervalo cleanup (padrão: 1) |

## Estrutura de Variáveis no Dokploy

```
Projeto (shared)
└── DATABASE_URL=${{pg_dex_account.CONNECTION_URI}}

Aplicação
└── DEX_JWT_SECRET=minha-chave
└── DEX_ALLOWED_ORIGINS=https://app.exemplo.com
```

## Troubleshooting

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
