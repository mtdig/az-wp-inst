# Vereisten – Installatiegids

Gedetailleerde installatie-instructies per besturingssysteem voor alle tools die nodig zijn om dit project te draaien.

## Inhoudsopgave

- [Vereisten – Installatiegids](#vereisten--installatiegids)
  - [Inhoudsopgave](#inhoudsopgave)
  - [Overzicht](#overzicht)
  - [Waarom uv voor Ansible?](#waarom-uv-voor-ansible)
  - [Windows](#windows)
    - [Optie 1 — WSL 2 (aanbevolen)](#optie-1--wsl-2-aanbevolen)
    - [Optie 2 — Native Windows](#optie-2--native-windows)
      - [Terraform](#terraform)
      - [Azure CLI](#azure-cli)
      - [Python](#python)
      - [uv](#uv)
      - [Make](#make)
      - [Ansible (via uv)](#ansible-via-uv)
  - [macOS (Apple Silicon)](#macos-apple-silicon)
  - [Linux – Debian / Ubuntu](#linux--debian--ubuntu)
  - [Linux – Arch](#linux--arch)
  - [Linux – Gentoo](#linux--gentoo)
  - [Linux – NixOS](#linux--nixos)
    - [Met flakes (aanbevolen)](#met-flakes-aanbevolen)
    - [Eenmalig (zonder flakes)](#eenmalig-zonder-flakes)
  - [FreeBSD](#freebsd)
  - [Config-starter (TUI)](#config-starter-tui)
    - [Optie 1 — Download van GitHub Releases (geen Go nodig)](#optie-1--download-van-github-releases-geen-go-nodig)
    - [Optie 2 — Zelf compileren (Go ≥ 1.21 vereist)](#optie-2--zelf-compileren-go--121-vereist)
    - [Gebruik](#gebruik)

---

## Overzicht

| Tool | Versie | Doel |
|---|---|---|
| **Terraform** | ≥ 1.5 | Azure infrastructuur provisioning |
| **Azure CLI** | latest | Authenticatie (`az login`) |
| **Python** | ≥ 3.12 | Runtime voor Ansible |
| **uv** | latest | Python dependency beheer (installeert Ansible in venv) |
| **Make** | any | Commando-orkestrator (Makefile) |
| **SSH** | any | Verbinding met de VM |
| **Go** | ≥ 1.21 | _Optioneel_ — alleen nodig om config-starter zelf te compileren |

---

## Waarom uv voor Ansible?

Ansible is een Python pakket. Je kunt het installeren met `pip install ansible`, maar dat heeft nadelen:

1. **Systeemvervuiling** — `pip install --user ansible` of `sudo pip install ansible` dumpt honderden bestanden in je systeem-Python. Bij een OS-upgrade of een ander project dat Ansible nodig heeft met een andere versie krijg je conflicten.
2. **Reproduceerbaarheid** — dit project heeft een `pyproject.toml` met een vastgepinde Ansible versie (`>=13.4.0`). Met `uv sync` krijgt iedereen exact dezelfde versie in een geïsoleerde virtuele omgeving.
3. **Snelheid** — `uv` is geschreven in Rust en installeert pakketten 10-100× sneller dan `pip`.
4. **Geen activatie nodig** — `uv run ansible-playbook ...` draait automatisch in de juiste venv zonder dat je die hoeft te activeren.

Concreet: `uv sync` leest `pyproject.toml`, maakt een `.venv/` map aan in de projectroot, installeert Ansible daarin, en het `Makefile` gebruikt `uv run` om alles in die venv uit te voeren. Je systeem-Python blijft schoon.

---

## Windows

> **Aanbevolen:** Gebruik [WSL 2](https://learn.microsoft.com/windows/wsl/install) met Ubuntu en volg dan de [Debian / Ubuntu](#linux--debian--ubuntu) instructies. Dat is veruit het makkelijkst.

### Optie 1 — WSL 2 (aanbevolen)

```powershell
# Installeer WSL (PowerShell als administrator)
wsl --install -d Ubuntu

# Start Ubuntu, volg dan de Debian/Ubuntu instructies hieronder
```

### Optie 2 — Native Windows

#### Terraform

Download de [Windows AMD64 binary](https://developer.hashicorp.com/terraform/install) en voeg het pad toe aan je `PATH`, of via winget:

```powershell
winget install Hashicorp.Terraform
```

#### Azure CLI

```powershell
winget install Microsoft.AzureCLI
```

#### Python

```powershell
winget install Python.Python.3.12
```

#### uv

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
```

#### Make

Installeer via [Chocolatey](https://chocolatey.org/install):

```powershell
choco install make
```

Of via winget met GnuWin32:

```powershell
winget install GnuWin32.Make
```

#### Ansible (via uv)

```powershell
cd opdracht4
uv sync
```

---

## macOS (Apple Silicon)

Alles gaat via [Homebrew](https://brew.sh/):

```bash
# Homebrew installeren (als je het nog niet hebt)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Terraform
brew tap hashicorp/tap
brew install hashicorp/tap/terraform

# Azure CLI
brew install azure-cli

# Python 3.12+
brew install python@3.12

# uv
brew install uv

# Make (zit al in macOS, maar voor nieuwste versie)
brew install make

# SSH zit standaard in macOS

# Ansible (via uv, in de projectmap)
cd opdracht4
uv sync
```

---

## Linux – Debian / Ubuntu

```bash
# Terraform — officiële HashiCorp repository
sudo apt-get update && sudo apt-get install -y gnupg software-properties-common
wget -O- https://apt.releases.hashicorp.com/gpg | \
  gpg --dearmor | sudo tee /usr/share/keyrings/hashicorp-archive-keyring.gpg > /dev/null
echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] \
  https://apt.releases.hashicorp.com $(lsb_release -cs) main" | \
  sudo tee /etc/apt/sources.list.d/hashicorp.list
sudo apt-get update && sudo apt-get install -y terraform

# Azure CLI
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Python 3.12+ en make
sudo apt-get install -y python3 python3-venv make

# uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# SSH
sudo apt-get install -y openssh-client

# Ansible (via uv, in de projectmap)
cd opdracht4
uv sync
```

---

## Linux – Arch

```bash
# Terraform
sudo pacman -S terraform

# Azure CLI (via AUR — gebruik yay, paru, of een andere AUR helper)
yay -S azure-cli

# Python, make, SSH
sudo pacman -S python make openssh

# uv
curl -LsSf https://astral.sh/uv/install.sh | sh
# of via pacman (als beschikbaar)
sudo pacman -S uv

# Ansible (via uv, in de projectmap)
cd opdracht4
uv sync
```

---

## Linux – Gentoo

```bash
# Terraform
sudo emerge --ask app-admin/terraform

# Azure CLI
sudo emerge --ask dev-util/azure-cli
# of via pip als het niet in de tree zit:
pip install --user azure-cli

# Python, make, SSH
sudo emerge --ask dev-lang/python dev-build/make net-misc/openssh

# uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Ansible (via uv, in de projectmap)
cd opdracht4
uv sync
```

---

## Linux – NixOS

Dit project bevat een `flake.nix` die automatisch Python en uv beschikbaar maakt.

### Met flakes (aanbevolen)

```bash
# Start de dev shell — installeert Python, uv, en draait uv sync automatisch
nix develop

# Terraform en Azure CLI via je system configuration.nix:
# environment.systemPackages = with pkgs; [ terraform azure-cli gnumake openssh ];
```

Voeg toe aan je `configuration.nix` of `home.nix`:

```nix
environment.systemPackages = with pkgs; [
  terraform
  azure-cli
  gnumake
  openssh
  uv
  python312
];
```

Dan `sudo nixos-rebuild switch`.

### Eenmalig (zonder flakes)

```bash
nix-shell -p terraform azure-cli gnumake python312 uv openssh
cd opdracht4
uv sync
```

---

## FreeBSD

```bash
# Terraform
sudo pkg install terraform

# Azure CLI (via pip, geen native port)
sudo pkg install python312 py312-pip
pip install --user azure-cli

# Make en SSH
sudo pkg install gmake openssh-portable

# uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Ansible (via uv, in de projectmap)
cd opdracht4
uv sync
```

> **Let op:** Gebruik `gmake` in plaats van `make` op FreeBSD. Draai alle `make` commando's als `gmake`:
> ```bash
> gmake init
> gmake all
> ```

---

## Config-starter (TUI)

De interactieve configuratie generator helpt je om `terraform.tfvars.json` en `ansible_vars.json` aan te maken.

### Optie 1 — Download van GitHub Releases (geen Go nodig)

Haal de laatste binary voor jouw platform op van de [releases pagina](https://github.com/mtdig/az-wp-inst/releases/latest):

| Platform | Bestand |
|---|---|
| Linux x86_64 | `config-starter-vX.X.X-linux-amd64` |
| Linux ARM64 | `config-starter-vX.X.X-linux-arm64` |
| macOS Apple Silicon | `config-starter-vX.X.X-darwin-arm64` |
| Windows x86_64 | `config-starter-vX.X.X-windows-amd64.exe` |

```bash
# Voorbeeld: Linux x86_64
curl -LO https://github.com/mtdig/az-wp-inst/releases/latest/download/config-starter-linux-amd64
chmod +x config-starter-linux-amd64
./config-starter-linux-amd64
```

### Optie 2 — Zelf compileren (Go ≥ 1.21 vereist)

```bash
# Go installeren (als je het nog niet hebt)
# macOS:   brew install go
# Debian:  sudo apt install golang-go  (of download van https://go.dev/dl/)
# Arch:    sudo pacman -S go
# NixOS:   nix-shell -p go
# FreeBSD: sudo pkg install go

# Compileer en draai
cd config-starter
make run

# Of alleen compileren -> bin/config-starter
make build

# Cross-compileer voor alle platformen -> bin/
make all
```

### Gebruik

Draai de config-starter **vanuit de `config-starter/` map** of de projectroot:

```bash
cd config-starter && make run
```

De TUI leidt je door alle instellingen en schrijft de JSON bestanden naar de projectroot.
