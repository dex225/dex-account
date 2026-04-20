# API Reference

**Base URL:** `/api/v1`

---

## Autenticação

### Login
```
POST /auth/login
```

**Body:**
```json
{
  "email": "admin@example.com",
  "password": "senha123"
}
```

**Resposta (sucesso sem 2FA):**
```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Resposta (2FA necessário):**
```json
{
  "challenge_token": "eyJ...",
  "expires_in": 300
}
```

---

### Verificar 2FA
```
POST /auth/verify-2fa
```

**Body:**
```json
{
  "challenge_token": "eyJ...",
  "code": "123456"
}
```

**Resposta:**
```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

---

### Refresh Token
```
POST /auth/refresh
```

**Headers:**
```
Cookie: refresh_token=<token>
```

**Resposta:**
```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

---

### Logout
```
POST /auth/logout
```

**Headers:**
```
Cookie: refresh_token=<token>
Authorization: Bearer <access_token>
```

**Resposta:**
```json
{
  "message": "Logged out"
}
```

---

## Recuperação de Senha

### Esqueci a Senha
```
POST /auth/password/forgot
```

**Body:**
```json
{
  "email": "usuario@example.com"
}
```

**Resposta:**
```json
{
  "message": "If the email exists, a reset link has been sent"
}
```

---

### Resetar Senha
```
POST /auth/password/reset
```

**Body:**
```json
{
  "token": "abc123...",
  "new_password": "novaSenha123"
}
```

**Resposta:**
```json
{
  "message": "Password reset successfully"
}
```

---

## 2FA

### Setup (requer autenticação)
```
POST /auth/2fa/setup
```

**Headers:**
```
Authorization: Bearer <access_token>
```

**Resposta:**
```json
{
  "totp_uri": "otpauth://totp/dex-account:email?secret=...",
  "secret": "JBSWY3DPEHPK3PXP"
}
```

---

### Habilitar (requer autenticação)
```
POST /auth/2fa/enable
```

**Headers:**
```
Authorization: Bearer <access_token>
```

**Body:**
```json
{
  "code": "123456"
}
```

**Resposta:**
```json
{
  "message": "2FA enabled successfully"
}
```

---

## Recuperação de Emergência

### Recover
```
POST /auth/emergency/recover
```

**Headers:**
```
X-Emergency-Key: <DEX_EMERGENCY_API_KEY>
```

**Body:**
```json
{
  "email": "admin@example.com"
}
```

**Resposta:**
```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 300
}
```

---

## Usuários

### Criar Usuário (requer autenticação + role Admin)
```
POST /users/create
```

**Headers:**
```
Authorization: Bearer <access_token>
```

**Body:**
```json
{
  "email": "novo@example.com",
  "password": "senha123"
}
```

**Resposta:**
```json
{
  "id": "uuid",
  "email": "novo@example.com",
  "is_2fa_enabled": false,
  "created_at": "2024-01-01T00:00:00Z"
}
```

---

### Meu Perfil (requer autenticação)
```
GET /users/me
```

**Headers:**
```
Authorization: Bearer <access_token>
```

**Resposta:**
```json
{
  "id": "uuid",
  "email": "admin@example.com",
  "is_2fa_enabled": true,
  "created_at": "2024-01-01T00:00:00Z"
}
```

---

## Health Checks

### Liveness
```
GET /health
```

**Resposta:** `200 OK`

---

### Readiness
```
GET /ready
```

**Resposta:** `200 OK` ou `500 Internal Server Error`

---

### Metrics (Prometheus)
```
GET /metrics
```

**Resposta:** `200 OK` com body no formato Prometheus exposition format

---

## Códigos de Erro

| HTTP Status | Erro |
|-------------|------|
| 400 | Bad Request |
| 401 | InvalidCredentials / InvalidToken / TokenExpired |
| 403 | Forbidden / UserInactive |
| 404 | UserNotFound |
| 429 | RateLimitExceeded |
| 500 | Internal Server Error |
