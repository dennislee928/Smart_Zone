# Multi-stage Dockerfile for ScholarshipOps API
# Stage 1: Build Rust scraper
FROM rust:1.75 as rust-builder
WORKDIR /app
COPY scripts/search_scholarships ./scripts/search_scholarships
WORKDIR /app/scripts/search_scholarships
RUN cargo build --release

# Stage 2: Build Go applications
FROM golang:1.23 as go-builder
WORKDIR /app
COPY scripts/schedule_applications ./scripts/schedule_applications
COPY scripts/track_progress ./scripts/track_progress
WORKDIR /app/scripts/schedule_applications
RUN go mod download && go build -o /app/bin/schedule_applications .
WORKDIR /app/scripts/track_progress
RUN go mod download && go build -o /app/bin/track_progress .

# Stage 3: Node.js runtime
FROM node:20-alpine
WORKDIR /app

# Install wrangler CLI globally
RUN npm install -g wrangler@4.54.0

# Copy built binaries
COPY --from=rust-builder /app/scripts/search_scholarships/target/release/search_scholarships /app/bin/
COPY --from=go-builder /app/bin/schedule_applications /app/bin/
COPY --from=go-builder /app/bin/track_progress /app/bin/

# Copy project files
COPY container/package.json container/package-lock.json* ./
COPY container/tsconfig.json ./
COPY container/vite.config.ts ./
COPY container/wrangler.jsonc ./
COPY container/drizzle.config.ts ./
COPY container/src ./src
COPY container/migrations ./migrations

# Copy tracking and config files (needed by scripts)
COPY tracking ./tracking
COPY Config ./Config
COPY tasks ./tasks

# Install dependencies
RUN npm install

# Expose port
EXPOSE 8787

# Set environment variables
ENV NODE_ENV=production
ENV ROOT=/app

# Start command
CMD ["wrangler", "dev"]
