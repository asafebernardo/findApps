# FindApps

Gerenciador universal de aplicativos para Linux. Detecta, organiza e desinstala programas instalados via **APT**, **DNF**, **Flatpak**, **Snap**, **AppImage** e instalações **manuais**, com interface nativa GTK4/libadwaita.

> MVP 0.1.0 — focado em descoberta e gerenciamento. Instalação e atualizações estão preparados na arquitetura, mas ainda não implementados.

## Distribuições suportadas

| Família | Prioridade | Backends típicos |
|---------|------------|------------------|
| Debian / Ubuntu e derivados | Alta | APT, Snap, Flatpak, AppImage, Manual |
| Fedora / RHEL e derivados | Alta | DNF, Flatpak, AppImage, Manual |
| Arch, openSUSE, Nix | Futuro | Arquitetura extensível pronta |

A interface **não** contém lógica de distribuição. Os backends são detectados automaticamente; backends ausentes são omitidos sem erro.

## Backends implementados

| Backend | Detectar | Listar | Desinstalar | Instalar / Atualizar |
|---------|----------|--------|-------------|----------------------|
| APT | Sim | Sim | Sim (pkexec) | Stub |
| DNF | Sim | Sim | Sim (pkexec) | Stub |
| Flatpak | Sim | Sim | Sim | Stub |
| Snap | Sim | Sim | Sim (pkexec) | Stub |
| AppImage | Sim | Sim | Sim (arquivos em `$HOME`) | Stub |
| Manual | Sim | Sim | Limitado (`~/.local`) | Stub |

## Dependências do sistema

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install build-essential pkg-config \
  libgtk-4-dev libadwaita-1-dev libglib2.0-dev \
  policykit-1
```

### Fedora

```bash
sudo dnf install gcc pkg-config \
  gtk4-devel libadwaita-devel glib2-devel \
  polkit
```

Também é necessário o [Rust](https://rustup.rs/) (1.75+ recomendado):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Desenvolvimento

```bash
# Clonar / entrar no projeto
cd findapps

# (recomendado) deps do sistema instaladas — ver seção acima
# Se os headers *-dev não puderem ser instalados via apt/dnf, use:
#   source scripts/dev-env.sh

# Executar em modo desenvolvimento
cargo run

# Compilar release
cargo build --release

# Binário
./target/release/findapps
```

### Variáveis úteis

```bash
RUST_LOG=debug cargo run   # logs detalhados no stderr e em ~/.local/share/findapps/logs/
```

## Testes

```bash
cargo test
```

Os testes usam **mocks** dos gerenciadores de pacotes e fixtures temporárias (AppImage/desktop). Não alteram o sistema real.

## Empacotamento

Esqueletos em `packaging/`:

### Snap (Ubuntu App Center)

Requer `snapcraft` e conta em [snapcraft.io](https://snapcraft.io) com o developer agreement assinado.

> **Importante:** o `snapcraft` falha se o caminho do projeto tiver espaços (ex.: `Área de trabalho`). Use o script abaixo, que empacota a partir de um diretório limpo.

```bash
# Registrar o nome (uma vez)
snapcraft login
snapcraft register findapps

# Empacotar (recomendado — evita bug do espaço no caminho)
./scripts/build-snap.sh

# Ou manualmente, a partir de um path sem espaços:
#   rsync -a --exclude target --exclude .git ./ ~/findapps-snap-build/
#   cd ~/findapps-snap-build && sudo snapcraft pack --destructive-mode

# Instalar localmente para teste (use o nome real do arquivo)
sudo snap install --dangerous --classic ./findapps_*.snap

# Publicar (após revisão classic pela Canonical)
snapcraft upload --release=edge ./findapps_*.snap
```

O snap usa `confinement: classic` porque precisa acessar gerenciadores do host (APT, Flatpak, Snap, pkexec).

### Flatpak

```bash
flatpak-builder --user --install build-dir packaging/flatpak/br.com.findapps.FindApps.yml
```

### .deb (esqueleto)

Arquivos em `packaging/deb/`. Integre com `dh` / cópia para `debian/` conforme o fluxo da distribuição.

### AppImage (esqueleto)

1. `cargo build --release`
2. Use `packaging/appimage/AppImageBuilder.yml` com [appimage-builder](https://appimage-builder.readthedocs.io/) ou linuxdeploy.

Arquivos desktop/metainfo/PolicyKit estão em `data/`:

- `br.com.findapps.FindApps.desktop`
- `br.com.findapps.FindApps.metainfo.xml`
- `br.com.findapps.FindApps.policy`

## Segurança

- Comandos com **argumentos separados** (sem shell).
- IDs de pacote validados antes de operações.
- Elevação apenas na operação (`pkexec` / PolicyKit), nunca `sudo` no processo inteiro.
- Confirmação explícita antes de desinstalar, com descrição clara do backend e da operação.

## Arquitetura

```text
                 FindApps
                     │
              PackageManager
                     │
       ┌─────────────┼─────────────┐
      APT          DNF          Flatpak
       │             │             │
      Snap        AppImage       Manual
```

Cada backend implementa a trait `PackageBackend` (`detect`, `list_installed`, `get_details`, `uninstall`, `install`, `update`, `check_updates`).

## Roadmap

- [ ] Instalação com escolha de método (Flatpak / Snap / APT / …)
- [ ] Detecção e aplicação de atualizações
- [ ] Backend Arch / pacman
- [ ] Backend openSUSE / zypper
- [ ] Backend Nix
- [ ] Pacotes RPM oficiais e AUR
- [x] i18n (English default + Chinese, Spanish, Hindi, Arabic, Portuguese, Russian)

## Licença

GPL-3.0-or-later
