# Runtime stage
# Usa o dist que já está buildado e commitado no repo
FROM node:alpine AS runner
WORKDIR /app
ENV NODE_ENV=production

COPY src/frontend/dist ./dist
COPY src/frontend/package*.json ./

EXPOSE 80

CMD ["npx", "serve", "-s", "dist", "-l", "80"]
