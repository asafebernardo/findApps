#!/usr/bin/env bash
# Ambiente de desenvolvimento quando os pacotes *-dev do sistema não estão instalados.
# Uso: source scripts/dev-env.sh && cargo run

PREFIX="${FINDAPPS_DEPS:-$HOME/.local/findapps-deps}"

if [[ ! -d "$PREFIX/usr/lib/x86_64-linux-gnu/pkgconfig" ]]; then
  echo "Aviso: $PREFIX não encontrado."
  echo "Instale as deps do sistema (ver README) ou extraia os pacotes -dev em $PREFIX."
fi

export PKG_CONFIG_PATH="$PREFIX/usr/lib/x86_64-linux-gnu/pkgconfig:$PREFIX/usr/share/pkgconfig:${PKG_CONFIG_PATH:-}"
export CPATH="$PREFIX/usr/include:${CPATH:-}"
export LIBRARY_PATH="$PREFIX/usr/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu:${LIBRARY_PATH:-}"
export RUSTFLAGS="-C link-arg=-L$PREFIX/usr/lib/x86_64-linux-gnu -C link-arg=-L/usr/lib/x86_64-linux-gnu -C link-arg=-lpcre2-8 ${RUSTFLAGS:-}"

echo "FindApps build env carregado (PKG_CONFIG_PATH aponta para headers locais se necessário)."
