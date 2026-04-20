# Documento de Design de Software (SDD)
**Projeto:** DEX Account (Serviço de Autenticação - IAM)
**Versão:** 1.0
**Contexto:** Fundação de identidade e controle de acesso (Role-Based Access Control) para o ecossistema modular de ferramentas de gestão da agência Digital Expansion.

---

## 1. Propósito e Escopo
A **DEX Account** é o "passaporte universal" da plataforma. Operando como um microsserviço de backend rigorosamente isolado, gerencia a identidade dos usuários, autenticação (login, senhas, TOTP) e emissão de tokens de acesso. O sistema é agnóstico em relação às regras de negócio das ferramentas finais (ex: precificação, ads). Seu único objetivo é garantir que apenas usuários verificados e autorizados acessem os módulos internos.

---

## 2. Decisões Arquiteturais Core
* **Segurança por Design (Closed System):** Por ser um sistema SaaS B2B interno, **não existe rota pública de registro**. A criação de contas é centralizada e restrita a administradores.
* **Isolamento de Estado:** As ferramentas da agência farão validação *stateless* (lendo a assinatura criptográfica do JWT). A DEX Account manterá o estado *stateful* das sessões de longo prazo no banco de dados para permitir revogações imediatas em caso de invasão.
* **Estratégia de Roles (MVP):** O banco de dados nascerá com as tabelas relacionais completas para RBAC (Role-Based Access Control), porém o *seed* inicial populartá exclusivamente a role `Admin`. Isso cria a fundação correta sem cair na armadilha da otimização prematura.

---

## 3. Stack Tecnológica Base
A seleção tecnológica foca em segurança *memory-safe*, latência em microssegundos e redução extrema da superfície de ataque.

* **Linguagem:** Rust (Compilação estática visando a target `x86_64-unknown-linux-musl`).
* **Framework Web:** Axum (Alta performance, tipagem forte para extração de rotas e integração nativa com o ecossistema assíncrono Tokio).
* **Banco de Dados:** PostgreSQL (Instância provisionada no Dokploy, isolada na rede Docker).
* **ORM / Driver:** SQLx. Utiliza validação de queries contra o banco de dados em tempo de compilação, operando em *offline mode* (`.sqlx/`) para viabilizar builds em CI/CD sem expor o banco de produção.
* **Criptografia:**
  * Hashes de Senha: `Argon2id`.
  * Hashes de Sessão (Banco): `SHA-256`.
* **Tokens e Sessão:** `jsonwebtoken` (Access Tokens) e controle manual de Cookies `HttpOnly` para o Refresh Token.
* **Segundo Fator (2FA):** `totp-rs` para geração de QR Codes e validação de senhas temporárias (Time-Based One-Time Password).
* **Observabilidade:** OpenTelemetry para tracing distribuído e métricas Prometheus.
* **Geração Aleatória:** `rand::rngs::OsRng` para CSPRNG (tokens, segredos, UUIDs).

---

## 4. Modelagem de Dados (PostgreSQL)
A modelagem utiliza estritamente **UUIDv7** para chaves primárias. Por possuir um timestamp embutido no início do hash, o UUIDv7 é ordenado cronologicamente, garantindo alta performance de indexação no Postgres (evitando a fragmentação comum do UUIDv4).

* **`users`**
    * `id` (UUIDv7, Primary Key)
    * `email` (VARCHAR, Unique, Indexed)
    * `password_hash` (VARCHAR)
    * `totp_secret` (VARCHAR, Nullable - populado ao gerar o QR Code)
    * `is_2fa_enabled` (BOOLEAN, Default: false - ativado apenas após a primeira validação bem-sucedida)
    * `is_active` (BOOLEAN, Default: true)
    * `created_at` / `updated_at` (TIMESTAMP)

* **`roles`**
    * `id` (UUIDv7, Primary Key)
    * `name` (VARCHAR, Unique)

* **`permissions`**
    * `id` (UUIDv7, Primary Key)
    * `name` (VARCHAR, Unique)

* **`role_permissions`** (Tabela de junção)
    * `role_id` (UUID, Foreign Key -> roles.id, ON DELETE CASCADE)
    * `permission_id` (UUID, Foreign Key -> permissions.id, ON DELETE CASCADE)
    * PRIMARY KEY (role_id, permission_id)

* **`user_roles`** (Tabela de junção)
    * `user_id` (UUID, Foreign Key -> users.id, ON DELETE CASCADE)
    * `role_id` (UUID, Foreign Key -> roles.id, ON DELETE CASCADE)
    * PRIMARY KEY (user_id, role_id)

* **`refresh_token_chains`**
    * `id` (UUIDv7, Primary Key)
    * `user_id` (UUID, Foreign Key -> users.id, ON DELETE CASCADE)
    * `chain_id` (UUID, NOT NULL - Agrupa tokens da mesma cadeia de RTR)
    * `token_hash` (VARCHAR, NOT NULL - Hash SHA-256 do token, nunca o texto claro)
    * `previous_token_hash` (VARCHAR, Nullable - Permite reverter a cadeia)
    * `created_at` (TIMESTAMP)
    * `expires_at` (TIMESTAMP)
    * `is_revoked` (BOOLEAN, Default: false)

* **`password_resets`**
    * `token_hash` (VARCHAR, Primary Key) - *Armazena o hash SHA-256 do token de recuperação.*
    * `user_id` (Foreign Key -> users.id)
    * `expires_at` (TIMESTAMP - Validade curta, ex: 30 minutos)

---

## 5. Fluxo de Segurança, Sessão e RTR
O sistema adota o padrão ouro de defesa em profundidade para Single Page Applications (SPAs).

1. **Access Token (JWT - Vida Curta):**
    * **Duração:** 15 minutos.
    * **Payload:** `{ sub: "user_id", role: "Admin", exp: timestamp }`.
    * **Transporte:** Trafega no corpo da resposta da API, fica na memória RAM do frontend e é enviado via Header `Authorization: Bearer <token>`.
2. **Refresh Token (Sessão Longa):**
    * **Duração:** 15 a 30 dias.
    * **Geração:** O Rust gera uma string aleatória de 64 bytes usando `OsRng` em Base64.
    * **Armazenamento no Banco:** Faz-se o hash (SHA-256) da string e salva-se *apenas o hash*. Dumps de banco não vazam sessões utilizáveis.
    * **Armazenamento no Cliente:** O token em texto claro viaja dentro de um Cookie com as flags `HttpOnly`, `Secure` e `SameSite=Strict`. O frontend não tem acesso de leitura a ele.
3. **Refresh Token Rotation (RTR) com Revogação Granular:**
    * Toda vez que o Access Token vence, o navegador envia o Cookie automaticamente para a rota `/refresh`.
    * O servidor faz o hash do token recebido, busca no banco (`refresh_token_chains`):
        * Se o token existe, é válido e não expirou: **deleta o token antigo, insere novo token na mesma cadeia (`chain_id`), emite novo Cookie com novo Refresh Token e devolve novo JWT**.
        * Se o token já foi utilizado (indicando clonagem): revoga **apenas a cadeia específica** (`chain_id`), não todas as sessões do usuário.
    * **Proteção contra Roubo:** O campo `previous_token_hash` permite rastreamento da cadeia. Tokens usados fora de ordem comprometem a cadeia e são revogados.

---

## 6. Autenticação de Dois Fatores (2FA) Opcional
Visando reduzir o atrito na adoção da plataforma, o 2FA é encorajado, porém opcional (Opt-in).

* **Habilitação:** O usuário escaneia o QR Code no dashboard. O `totp_secret` é salvo no banco, mas a flag `is_2fa_enabled` permanece `false` até que ele digite o primeiro código de 6 dígitos com sucesso, provando que o app autenticador está sincronizado.
* **Impacto no Login:** O endpoint `/login` verifica as credenciais. Se `is_2fa_enabled == true`, ele aborta a entrega dos tokens finais, retorna um token temporário de desafio (ex: 5 minutos de validade) e exige a chamada subsequente ao endpoint de verificação.

---

## 7. Notificações e Recuperação de Senha
Para manter a DEX Account purista e sem dependências lentas, o envio de e-mails de recuperação é isolado da API principal.

* **Microsserviço de E-mail (Mail Container):** Um contêiner independente na rede Docker da infraestrutura.
* **Integração:** Quando um usuário aciona o "Esqueci a Senha", a DEX Account gera um token seguro (com hash no banco), e faz um disparo rápido via HTTP (ou protocolo interno) para o Mail Container informando a intenção.
* **Vantagem:** Evita lentidão na API principal aguardando timeouts de servidores SMTP. Facilita a futura troca de provedor de envios.

---

## 8. Recuperação de Emergência
Prevenção contra "lockout" total do sistema caso o administrador perca o celular com o 2FA e a senha ao mesmo tempo.

### A) Endpoint de Emergência Remota
* **Rota:** `POST /api/v1/auth/emergency/recover`
* **Proteção:** Header `X-Emergency-Key` contendo a `DEX_EMERGENCY_API_KEY`.
* **Ação:** Gera um JWT temporário de 5 minutos com role `Admin` para o e-mail especificado no body.
* **Não modifica:** Senha, TOTP secret, ou qualquer dado do usuário.
* **Logging:** Registra IP de origem, timestamp, e-mail alvo e resultado em `audit_log`.

### B) CLI de Recuperação Local
* **Binário:** `dex-account-recovery` (binário separado ou flag `--emergency-recover`)
* **Execução:** Via `docker exec` no Dokploy
* **Variáveis de ambiente necessárias:**
    * `DATABASE_URL` (conexão direta ao Postgres)
    * `DEX_ADMIN_EMAIL` (e-mail do admin a recuperar)
    * `DEX_JWT_SECRET` (para assinar o token de emergência)
* **Ação:** Conecta direto ao banco, gera token JWT temporário de emergência (15 min), exibe no stdout.
* **Pós-uso:** Administrador deve fazer login e desativar a flag de emergência.

### C) Protocolo Operacional
1. Acessar Dokploy em ambiente seguro (VPN).
2. Obter `DEX_EMERGENCY_API_KEY` do cofre de senhas.
3. Chamar endpoint ou executar CLI.
4. Fazer login com JWT temporário.
5. Recuperar acesso ao 2FA ou desativá-lo.
6. Remover variável de ambiente e reiniciar contêiner para desabilitar o mecanismo.

---

## 9. Proteção de Rede e Camada de Transporte (Mitigação de Ataques)
A infraestrutura será configurada para bloquear comportamentos anômalos e interceptações antes que alcancem as regras de negócio.

* **HTTPS/TLS Obrigatório:** O sistema depende da flag `Secure` nos cookies, que não opera em conexões não-criptografadas. O Traefik (via Dokploy) gerenciará os certificados Let's Encrypt e forçará o redirecionamento de tráfego `HTTP -> HTTPS` em 100% das requisições.
* **CORS Rigoroso (Cross-Origin Resource Sharing):** O servidor Axum bloqueará qualquer requisição originada de domínios não autorizados. A configuração aceitará origens explícitas (ex: `https://myaccount.dex.com.br`, `https://app.dex.com.br`) e definirá `Access-Control-Allow-Credentials: true` para permitir o fluxo dos cookies `HttpOnly` exclusivamente para esses domínios. Nenhuma rota utilizará o wildcard `*`.
* **Rate Limiting (Proteção contra Força Bruta):** Implementação de *middleware* (como `tower-governor`) para limitar requisições por IP.
  * **Rotas Críticas:** O endpoint `/verify-2fa` terá um limite rigoroso (ex: bloqueio por 15 minutos após 5 tentativas incorretas) para inviabilizar adivinhação do PIN de 6 dígitos. O endpoint `/login` seguirá padrão similar.
  * **Prevenção de Spam:** A rota `/password/forgot` também será limitada para evitar sobrecarga no Microsserviço de E-mail.
* **Health Checks:**
  * `GET /health` (Liveness): Retorna 200 se o processo está rodando.
  * `GET /ready` (Readiness): Verifica conexão com banco de dados antes de aceitar tráfego.

---

## 10. Contrato da API (Endpoints)
Prefixo Base: `/api/v1/auth`

**Gestão Interna (Acesso Privado):**
* `POST /users/create`: Cria novos usuários. Exige Header com JWT válido e role `Admin`.
* `GET /users/me`: Retorna dados de perfil do usuário logado baseado no JWT.

**Autenticação e Sessão:**
* `POST /login`: Recebe JSON com e-mail/senha.
* `POST /verify-2fa`: Recebe token de desafio e código de 6 dígitos. Efetiva o login.
* `POST /refresh`: Lê o Cookie silenciosamente, rotaciona a sessão no banco e devolve um novo JWT.
* `POST /logout`: Recebe requisição autenticada, revoga o token no banco de dados e comanda a expiração do Cookie no navegador do cliente.

**Recuperação e 2FA:**
* `POST /password/forgot`: Inicia o fluxo de recuperação via e-mail.
* `POST /password/reset`: Recebe o token da URL do e-mail e a nova senha.
* `POST /2fa/setup`: Exige JWT. Gera o secret e devolve a URI do QR Code.
* `POST /2fa/enable`: Exige JWT e código de confirmação. Muda o status para ativado.

**Emergência:**
* `POST /emergency/recover`: Gera JWT temporário. Exige header `X-Emergency-Key`.

---

## 11. Infraestrutura, CI/CD e Deploy (Dokploy)
Arquitetura focada em isolamento de recursos e blindagem de contêineres.

* **Docker Multi-stage Build:**
  * **Estágio 1 (Builder):** Usa imagem oficial `rust`. Realiza o cache das dependências do Cargo. Utiliza o `.sqlx/` para validar as queries SQL offline. Compila estaticamente o binário para a target `musl`.
  * **Estágio 2 (Runtime):** Utiliza imagem `scratch` (completamente vazia, 0 bytes iniciais). O binário do Rust é o único artefato transferido.
* **Resultado Físico:** O contêiner de produção final pesará entre 10MB e 20MB, consumirá em média de 10 a 20MB de RAM.
* **Segurança de Contêiner:** Sendo uma imagem `scratch`, não existe `bash`, `sh`, `curl` ou qualquer binário do Linux embarcado. Invasões de sistema operacional base são impossibilitadas por design estrutural.

---

## 12. Migrations
* **Ferramenta:** `sqlx-cli` com `sqlx migrate`
* **Diretório:** `migrations/` na raiz do projeto
* **Convenção:** Formato `YYYYMMddHHMMSS_descricao` (ex: `20240101000000_initial_schema`)
* **Primeira migration:** `001_initial_schema` - Cria todas as tabelas do SDD
* **CI/CD:** Validação automática das migrations em pipelines

---

## 13. Observabilidade (OpenTelemetry)
* **Tracing Distribuído:**
  * Biblioteca: `tracing` + `opentelemetry` + `tracing-opentelemetry`
  * Spans para: login, 2fa, refresh, logout, emergency-recover
  * Attributes: user_id (hashed paraanonimizado), IP, user-agent, status_code
  * Exportação: OTLP para coletor configurável (Jaeger, Tempo, etc.)
* **Métricas Prometheus:**
  * Biblioteca: `metrics` crate
  * Counters: `auth_login_total`, `auth_login_failed_total`, `auth_2fa_attempts_total`
  * Histograms: `auth_refresh_latency_ms`, `auth_login_latency_ms`
  * Endpoint: `GET /metrics` (formato Prometheus)
* **Health & Readiness:**
  * `GET /health`: Liveness probe (processo ativo)
  * `GET /ready`: Readiness probe (DB reachable via `sqlx::connect().await`)

---

## 14. Logging
* **Formato:** JSON estruturado (serde_json)
* **Campos obrigatórios:** `timestamp` (ISO 8601), `level` (ERROR|WARN|INFO|DEBUG), `span`, `message`
* **Campos contextuais:** `user_id` (hash quando disponível), `ip`, `request_id` (UUIDv7 gerado por middleware)
* **Campos proibidos (nunca logar):** Senhas, tokens, refresh_tokens, totp_secret, api_keys
* **Níveis:**
  * `ERROR`: Falhas de autenticação suspeitas, erros de banco, panics
  * `INFO`: Login/logout bem-sucedidos, operações de admin
  * `DEBUG`: Detalhes de requisição (sem dados sensíveis)

---

## 15. Cleanup de Sessões Expiradas
* **Mecanismo:** Background task interna via Tokio spawn
* **Execução:** A cada 1 hora (configurável via `DEX_CLEANUP_INTERVAL_HOURS`)
* **Query:** `DELETE FROM refresh_token_chains WHERE expires_at < NOW() AND is_revoked = true`
* **Logging:** INFO com contagem de tokens removidos
* **Edge case:** Tokens expirados mas não revogados são mantidos para auditoria; apenas tokens revogados + expirados são deletados

---

## 16. Backup e Restore
* **Backup:** Via Dokploy (pg_dump agendado)
  * Frequência: diária com retenção de 30 dias
  * Armazenamento: Volume persistido ou object storage externo
* **Restore:**
  1. Parar contêiner da aplicação
  2. `psql $DATABASE_URL < backup.sql`
  3. Reiniciar aplicação
* **Testes:** Restore procedure testado trimestralmente

(End of file - total 297 lines)
