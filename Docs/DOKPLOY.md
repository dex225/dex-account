# Configuração no Dokploy - Docker Compose

## 1. Pré-requisitos

- Servidor Dokploy instalado e configurado
- Repositório Git conectado ao Dokploy (GitHub)
- Domínios configurados no DNS apontando para o servidor

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

## 4. Criar o Serviço Docker Compose

### Passo a Passo:

1. Dentro do projeto, clique em **Create Service** (não Create Application)
2. Selecione **Compose Type: Docker Compose**
3. Configure:

#### Configurações Gerais

- **Name:** `dex-account`
- **Repository:** `https://github.com/dex225/dex-account`
- **Branch:** `main`
- **Compose Path:** `./docker-compose.yml`

#### Variáveis de Ambiente

No Dokploy, defina as variáveis no nível do serviço:

| Variável | Descrição |
|----------|----------|
| `DATABASE_URL` | `${{pg_dex_account.CONNECTION_URI}}` |
| `DEX_JWT_SECRET` | Sua chave secreta (mín. 32 caracteres) |
| `DEX_EMERGENCY_API_KEY` | Chave para recuperação de emergência |
| `DEX_ALLOWED_ORIGINS` | `https://myaccount.agenciadex.com` |
| `DEX_AUTO_MIGRATE` | `false` (para produção) |

### Configurar Domínios

Após o deploy, configure os domínios na aba **Domains**:

1. Clique em **Add Domain**
2. Configure cada serviço:

| Serviço | Domínio | Porta |
|---------|---------|-------|
| `api` | `api.agenciadex.com` | 3000 |
| `frontend` | `myaccount.agenciadex.com` | 80 |

**Importante:** Marque **HTTPS** para cada domínio (Let's Encrypt automático).

### Preview Compose

Use o botão **Preview Compose** para ver como o Dokploy modificará seu arquivo antes do deploy.

## 5. Estrutura do docker-compose.yml

O arquivo `docker-compose.yml` na raiz do projeto:

```yaml
services:
  api:
    build:
      context: .
      dockerfile: Dockerfile
    expose:
      - 3000
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - DEX_JWT_SECRET=${DEX_JWT_SECRET}
      - DEX_EMERGENCY_API_KEY=${DEX_EMERGENCY_API_KEY}
      - DEX_ALLOWED_ORIGINS=${DEX_ALLOWED_ORIGINS}
      - DEX_AUTO_MIGRATE=${DEX_AUTO_MIGRATE:-false}
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped
    networks:
      - dokploy-network

  frontend:
    build:
      context: .
      dockerfile: src/frontend/Dockerfile
    expose:
      - 80
    environment:
      - VITE_API_TARGET=https://api.agenciadex.com
    depends_on:
      - api
    restart: unless-stopped
    networks:
      - dokploy-network

networks:
  dokploy-network:
    external: true
```

### O Dokploy Adiciona Automaticamente:

Quando você configura domínios na UI, o Dokploy automaticamente:
- Adiciona labels do Traefik
- Adiciona a rede `dokploy-network` aos serviços
- Configura SSL/TLS

**Importante:** Não adicione manualmente a rede `webgateway` - ela não existe no Dokploy.

## 6. Build do Frontend (IMPORTANTE)

### Variáveis de Ambiente Build-time vs Runtime

**Backend (Rust):** Lê variáveis em **tempo de execução** - quando o programa inicia, ele busca `std::env::var("VAR_NAME")`. Mudanças no Docker Compose funcionam imediatamente.

**Frontend (Vite/React):** As variáveis `VITE_*` são lidas e "hardcoded" no bundle JavaScript durante o **build**. Quando você roda `npm run build`, o Vite substitui todas as referências `import.meta.env.VITE_API_TARGET` pelo valor real naquele momento.

```
npm run build (build time)
├── VITE_API_TARGET=https://api.agenciadex.com  ← valor lido aqui
└── output: index.js com "https://api.agenciadex.com" hardcoded
```

Depois disso, mudar a variável no Docker Compose **não faz diferença nenhuma**.

### Passos para Rebuildar o Frontend

Se você mudar `VITE_API_TARGET`, precisa rebuildar locally e commitar o novo `dist/`:

```bash
# 1. Vá para o diretório do frontend
cd src/frontend

# 2. Build com a variável correta
VITE_API_TARGET=https://api.agenciadex.com npm run build

# 3. Isso gera novos arquivos em src/frontend/dist/

# 4. Commit e push
cd ../..
git add src/frontend/dist/
git commit -m "fix: update API URL in frontend bundle"
git push
```

## 7. Primeira vez - Rodar Migrations

Para o primeiro deploy, você pode habilitar migrations automáticas:

1. Adicione a variável: `DEX_AUTO_MIGRATE=true`
2. Deploy o serviço
3. Após migrations rodarem, mude para `DEX_AUTO_MIGRATE=false`
4. Redeploy

## 8. Estrutura de Variáveis no Dokploy

```
Projeto (compartilhado)
└── DATABASE_URL=${{pg_dex_account.CONNECTION_URI}}

Serviço Compose
├── DEX_JWT_SECRET=sua-chave-32-caracteres
├── DEX_EMERGENCY_API_KEY=sua-chave-emergencia
└── DEX_ALLOWED_ORIGINS=https://myaccount.agenciadex.com
```

## 9. Como o Rate Limiting Funciona

O backend usa `tower-governor` com `SmartIpKeyExtractor`, que lê:
- `X-Forwarded-For`
- `X-Real-IP`
- Fallback para IP direto

**Importante:** Para que o rate limiting funcione corretamente atrás do Traefik:
1. O Traefik deve enviar os headers `X-Forwarded-For` ou `X-Real-IP`
2. No Dokploy, isso é configurado automaticamente pelo Dokploy

## 10. Monitoramento

Cada serviço pode ser monitorado separadamente:
- Logs: disponível na aba **Logs**
- Métricas: Prometheus exporter na porta 3001 (API)

## 11. Variáveis de Ambiente Resumidas

| Variável | Obrigatório | Descrição |
|----------|-------------|-----------|
| `DATABASE_URL` | Sim | Connection string PostgreSQL |
| `DEX_JWT_SECRET` | Sim | Segredo JWT (mín. 32 chars) |
| `DEX_EMERGENCY_API_KEY` | Sim | Chave de emergência |
| `DEX_ALLOWED_ORIGINS` | Sim | URLs CORS |
| `DEX_AUTO_MIGRATE` | Não | Executa migrations automaticamente |

## 12. Troubleshooting

### Container não inicia

1. Verificar logs na aba **Logs**
2. Verificar se variáveis de ambiente estão corretas
3. Verificar se banco de dados está acessível

### Erro de conexão banco

1. Confirmar `DATABASE_URL` correto
2. Verificar se banco está na mesma rede Docker

### Frontend retorna 502

1. Verificar se o container `frontend` está rodando
2. Verificar logs do container frontend
3. Confirmar que o domínio está apontando para a porta 80

### API retorna 404 mas container está rodando

**Causa comum:** O container da API não tem `curl` instalado, então o healthcheck falha.

O Dockerfile da API (runtime stage) precisa ter curl:
```dockerfile
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists
```

Sem curl, o healthcheck `curl -f http://localhost:3000/health` falha, e o Traefik não consegue detectar o container.

### Frontend chama localhost:3000 em vez da API correta

**Causa:** O frontend foi buildado com a variável `VITE_API_TARGET` errada ou não foi rebuildado após mudança.

O valor de `VITE_API_TARGET` é hardcoded no bundle JavaScript no momento do build. Para corrigir:
1. Rebuild o frontend localmente com o valor correto
2. Commit e push do novo `dist/`

### CORS errors

Garantir que `DEX_ALLOWED_ORIGINS` contém exatamente as URLs do frontend, sem espaços.

## 13. Segurança em Produção

### Variáveis Sensíveis

- Nunca commit variáveis com senhas/secrets
- Usar variáveis do Dokploy
- Rotacionar `DEX_EMERGENCY_API_KEY` periodicamente

### Network

- O Dokploy adiciona automaticamente a rede `dokploy-network`
- Não exponha portas do banco para internet

### Health Checks

O health check configurado:
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
  interval: 30s
  timeout: 10s
  retries: 3
```

## 14. Problemas Conhecidos e Soluções

| Problema | Solução |
|----------|---------|
| Healthcheck falha com "curl not found" | Adicionar `curl` ao Dockerfile da API |
| Traefik retorna 404 para API | Verificar se container está na network `dokploy-network` e se curl está instalado |
| Frontend chama localhost:3000 | Rebuild o frontend com `VITE_API_TARGET` correto e commitar dist/ |
| Deploy falha com "webgateway not found" | Remover `webgateway` do docker-compose.yml (não existe no Dokploy) |
| Rate limiting não funciona | Usar `SmartIpKeyExtractor` ao invés de `PeerIpKeyExtractor` |

---

**Suporte:** Para dúvidas, consulte a documentação oficial do Dokploy em https://docs.dokploy.com ou entre no Discord.