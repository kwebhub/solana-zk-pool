# ============================================================
# Merkle service (Node.js + Fastify + noir_js)
# ============================================================

# ---------- Stage 1: builder ----------
FROM node:24-slim AS builder

WORKDIR /build

RUN corepack enable && corepack prepare pnpm@12 --activate

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile --prod

# ---------- Stage 2: runtime ----------
FROM node:24-slim

WORKDIR /app

RUN corepack enable && corepack prepare pnpm@12 --activate

COPY --from=builder /build/node_modules ./node_modules

COPY package.json ./
COPY src ./src
COPY circuits ./circuits

RUN useradd -r -u 1001 -s /bin/false appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 4003

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD node -e "fetch('http://localhost:4003/health').then(r => process.exit(r.ok ? 0 : 1)).catch(() => process.exit(1))"

CMD ["node", "src/server.js"]
