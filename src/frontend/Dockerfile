# Build stage
FROM node:alpine AS builder
WORKDIR /app

# Context is root (.), so we need to path to frontend/
COPY src/frontend/package*.json ./src/frontend/
WORKDIR /app/src/frontend
RUN npm ci

COPY src/frontend/ ./src/frontend/
RUN npm run build

# Runtime stage
FROM node:alpine AS runner
WORKDIR /app
ENV NODE_ENV=production

COPY --from=builder /app/src/frontend/dist ./dist
COPY --from=builder /app/src/frontend/package*.json ./

RUN npm ci --omit=dev

EXPOSE 80

CMD ["npx", "serve", "-s", "dist", "-l", "80"]
