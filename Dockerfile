# Etapa de Construção
    FROM --platform=linux/arm64 rust:1.81 AS builder

    # Instalar dependências necessárias
    RUN apt-get update && apt-get install -y musl-tools libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

    # Definir o diretório de trabalho
    WORKDIR /app

    # Copiar arquivos do projeto
    COPY . .

    # Adicionar target musl
    RUN rustup target add aarch64-unknown-linux-musl

    # Configurar variáveis de ambiente para OpenSSL
    ENV OPENSSL_DIR=/usr/lib/ssl
    ENV OPENSSL_LIB_DIR=/usr/lib/ssl/lib
    ENV OPENSSL_INCLUDE_DIR=/usr/lib/ssl/include

    # Compilar a aplicação em modo release para musl
    RUN cargo build --release --target aarch64-unknown-linux-musl

    # Strip do binário para reduzir o tamanho
    RUN strip target/aarch64-unknown-linux-musl/release/google-drive-checker

    # Etapa Final
    FROM --platform=linux/arm64 alpine:latest

    # Instalar dependências necessárias, bash e nano
    RUN apk add --no-cache ca-certificates bash nano

    # Definir o diretório de trabalho
    WORKDIR /app

    # Copiar o binário compilado da etapa de build
    COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/google-drive-checker .

    # Expor a porta da aplicação
    EXPOSE 3000

    # Tornar o shell padrão como bash (opcional)
    SHELL ["/bin/bash", "-c"]

    # Executar a aplicação diretamente
    CMD ["./google-drive-checker"]
