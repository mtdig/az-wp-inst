# Opdracht 4 – WordPress op Azure

Volledig geautomatiseerde deployment van een WordPress stack op Azure met **Terraform** voor provisioning en **Ansible** voor configuratiebeheer.  We gebruiken **Makefile** om deze uit te voeren.


## Wat wordt er aangemaakt

| Laag | Tool | Resources |
|---|---|---|
| **Infrastructuur** | Terraform | Resource Group, VNet, Subnet, NSG, Publiek IP, NIC, Ubuntu 22.04 VM, MySQL Flexible Server, firewallregels, auto-shutdown schema |
| **Configuratie** | Ansible | SSH hardening, UFW, fail2ban, Apache + PHP, WordPress, WP-CLI, remote MySQL database & gebruiker via SSL |

## Vereisten

| Vereiste | Opmerkingen |
|---|---|
| [Terraform](https://developer.hashicorp.com/terraform/install) ≥ 1.5 | Infrastructuur provisioning |
| [Azure CLI](https://learn.microsoft.com/cli/azure/install-azure-cli) | Authenticatie (`az login`) |
| [uv](https://astral.sh/uv) | Python/Ansible dependency beheer |
| SSH sleutelpaar | Standaard: `~/.ssh/id_ed25519_hogent` |
| [Make](https://makefiletutorial.com/) | Makefile command runner |


Op **NixOS** kan je de dev shell opstarten met `nix develop`.

## Snel aan de slag

```bash
# 1. Log in bij Azure, opent browser voor login
az login

# 2. Stel je subscription ID in via provisioning/terraform.tfvars

# 3. Deploy alles
export MYSQL_PASS="JouwVeiligWachtwoord123!"
make init
make all
```


Dat is alles. WordPress draait op het publieke IP van de VM.

## Make targets

Voer `make` of `make help` uit om alle targets te zien:

| Target | Beschrijving |
|---|---|
| `make init` | Terraform initialiseren (providers downloaden) |
| `make plan` | Bekijk wat Terraform zou aanmaken/wijzigen |
| `make apply` | Alle Azure infrastructuur aanmaken |
| `make configure` | Ansible playbook uitvoeren (leest automatisch Terraform outputs) |
| `make all` | **`apply` + `configure`** in één keer |
| `make info` | Huidige Terraform outputs tonen (IPs, FQDNs, …) |
| `make destroy` | Alle Azure resources verwijderen |
| `destroy-vm` | Enkel de VM en dependencies verwijderen (netwerk, compute) |
| `make clean` | Lokale Terraform state & cache opruimen |

### Secrets doorgeven

Het MySQL admin wachtwoord **moet** meegegeven worden. De SSH publieke sleutel wordt automatisch gelezen van `~/.ssh/id_ed25519_hogent.pub`.

```bash
# Optie A – inline
make apply MYSQL_PASS="JouwVeiligWachtwoord123!"

# Optie B – omgevingsvariabele
export MYSQL_PASS="JouwVeiligWachtwoord123!"
make all

# Optie C – Terraform omgevingsvariabele (werkt ook)
export TF_VAR_mysql_admin_password="JouwVeiligWachtwoord123!"
make all
```

### SSH sleutel aanpassen

```bash
make apply SSH_KEY=~/.ssh/mijn_andere_sleutel MYSQL_PASS="..."
```

## Hoe werkt het

```
make all
  │
  ├─ make apply          ← Terraform maakt Azure resources aan
  │   └─ outputs: public_ip_address, mysql_fqdn, admin_username, …
  │
  └─ make configure      ← Ansible configureert de VM
      ├─ leest automatisch Terraform outputs
      ├─ verbindt via SSH naar het publieke IP van de VM
      └─ geeft MySQL FQDN + admin login door als extra vars
```

Terraform outputs worden bij configure-time gelezen en via `-e` extra vars en dynamische inventory in de Ansible run geïnjecteerd. Geen handmatig kopiëren van IPs of hostnamen nodig.

## Projectstructuur

```
opdracht4/
├── Makefile                     # Orkestreeert alles
├── .gitignore
├── flake.nix                    # NixOS dev shell
├── pyproject.toml / uv.lock    # Python/Ansible dependencies
│
├── provisioning/                # Terraform root
│   ├── main.tf
│   ├── variables.tf
│   ├── outputs.tf
│   ├── terraform.tfvars
│   ├── README.md
│   └── modules/
│       ├── network/             # VNet, Subnet, NSG, Publiek IP, NIC
│       ├── compute/             # Ubuntu VM + auto-shutdown
│       └── database/            # MySQL Flexible Server + firewallregels
│
├── configuration_management/    # Ansible root
│   ├── ansible.cfg
│   ├── inventory.yml
│   ├── vault.yml                # Versleutelde geheimen
│   ├── README.md
│   ├── playbooks/
│   │   └── site.yml
│   └── roles/
│       ├── common/              # SSH, UFW, fail2ban
│       ├── mysql_client/        # MySQL client, remote DB/gebruiker aanmaak
│       └── wordpress/           # Apache, PHP, WordPress, WP-CLI
│
└── devops/                      # Originele ARM templates (ter referentie)
    ├── mysql/
    └── ubuntu/
```

## Na deployment

`make configure` werkt automatisch je lokale `~/.ssh/config` bij met een `azosboxes` alias (er wordt eerst een backup gemaakt naar `~/.ssh/config.bak`). Daarna kan je eenvoudig verbinden:

```bash
# Outputs bekijken
make info

# SSH naar de VM (via automatisch aangemaakte alias)
ssh azosboxes

# Of handmatig
ssh osboxes@$(cd provisioning && terraform output -raw public_ip_address)

# WordPress openen
# Open: https://sel-opdracht4.groep99.be
```

## Opruimen

```bash
make destroy MYSQL_PASS="JouwVeiligWachtwoord123!" 
```
_* of een ander complex wachtwoord, dit wordt niet gebruikt, maar de complixiteit ervan wordt wel gecontroleerd_