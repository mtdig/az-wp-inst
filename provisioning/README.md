# Opdracht 4 – WordPress Infrastructuur (Terraform)

Terraform configuratie die een volledige WordPress hosting stack op Azure provisioneert:

| Module | Resources |
|---|---|
| **network** | VNet, Subnet, NSG (SSH / HTTP / HTTPS), Publiek IP, NIC |
| **compute** | Ubuntu 22.04 LTS VM met auto-shutdown schema |
| **database** | MySQL 8.0 Flexible Server met firewallregels |

## Vereisten

| Tool | Minimale versie |
|---|---|
| [Terraform](https://developer.hashicorp.com/terraform/install) | >= 1.5.0 |
| [Azure CLI](https://learn.microsoft.com/cli/azure/install-azure-cli) | laatste versie |

Je hebt ook nodig:

- Een Azure abonnements-ID
- Een SSH sleutelpaar (`~/.ssh/id_rsa` / `~/.ssh/id_rsa.pub`)

## Snel aan de slag

### 1. Authenticeer bij Azure

```bash
az login
```

### 2. Configureer je abonnement

Bewerk `terraform.tfvars` en vervang de placeholder:

```hcl
subscription_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
```

### 3. Initialiseer Terraform

```bash
cd provisioning
terraform init
```

### 4. Plan & apply

```bash
terraform plan \
  -var="mysql_admin_password=JouwVeiligWachtwoord123!" \
  -var="admin_public_key=$(cat ~/.ssh/id_rsa.pub)"

terraform apply \
  -var="mysql_admin_password=JouwVeiligWachtwoord123!" \
  -var="admin_public_key=$(cat ~/.ssh/id_rsa.pub)"
```

### 5. Bekijk de outputs

```bash
terraform output
```

Belangrijkste outputs: `public_ip_address`, `mysql_fqdn`, `vm_name`.

### 6. Verbind met de VM

```bash
ssh osboxes@$(terraform output -raw public_ip_address)
```

### 7. Verwijder als je klaar bent

```bash
terraform destroy \
  -var="mysql_admin_password=JouwVeiligWachtwoord123!" \
  -var="admin_public_key=$(cat ~/.ssh/id_rsa.pub)"
```

## Geheimen beheren

Twee waarden zijn **gevoelig** en mogen nooit gecommit worden naar Git:

| Variabele | Wat is het | Hoe doorgeven |
|---|---|---|
| `admin_public_key` | SSH publieke sleutel voor de VM | `-var="admin_public_key=$(cat ~/.ssh/id_rsa.pub)"` |
| `mysql_admin_password` | MySQL admin wachtwoord | `-var="mysql_admin_password=..."` |

### Optie A – CLI vlaggen (hierboven getoond)

Geef geheimen mee met `-var` bij elke `plan` / `apply` / `destroy` aanroep. Eenvoudigste aanpak voor lokale ontwikkeling.

### Optie B – Omgevingsvariabelen

Exporteer ze zodat Terraform ze automatisch oppikt:

```bash
export TF_VAR_admin_public_key="$(cat ~/.ssh/id_rsa.pub)"
export TF_VAR_mysql_admin_password="JouwVeiligWachtwoord123!"

terraform apply          # geen -var vlaggen nodig
```

Voeg de exports toe aan je shell profiel (`.bashrc` / `.zshrc`) of een lokaal `.envrc` bestand (met [direnv](https://direnv.net/)) zodat je ze niet telkens opnieuw moet invoeren.

### Optie C – `secret.tfvars` (genegeerd door git)

Maak een bestand aan dat al in `.gitignore` staat:

```bash
cat > secret.tfvars <<'EOF'
admin_public_key     = "ssh-rsa AAAA..."
mysql_admin_password = "JouwVeiligWachtwoord123!"
EOF
```

Verwijs er dan naar:

```bash
terraform apply -var-file="secret.tfvars"
```

> **⚠️ Commit nooit `secret.tfvars`, `*.tfstate`, of `*.tfstate.*` naar versiebeheer.** De meegeleverde `.gitignore` dekt dit al af.

## Statebeheer

Terraform state wordt **lokaal** opgeslagen in `terraform.tfstate`. Dit bestand bevat resource-ID's en gevoelige outputs.

- Commit het **niet** naar Git (afgedekt door `.gitignore`).
- Maak een back-up als je alleen werkt.
- Voor teamgebruik, overweeg migratie naar een remote backend (bv. Azure Storage Account).

## Projectstructuur

```
opdracht4/
├── .gitignore                   # Houdt state & geheimen buiten Git
├── devops/                      # Originele ARM templates (enkel ter referentie)
│   ├── mysql/
│   └── ubuntu/
└── provisioning/                # ← Terraform root (voer commando's hier uit)
    ├── main.tf                  # Provider, resourcegroep, module-aanroepen
    ├── variables.tf             # Alle invoervariabelen met standaardwaarden
    ├── outputs.tf               # Doorgesluisde outputs van modules
    ├── terraform.tfvars         # Niet-gevoelige overrides (abonnements-ID)
    ├── README.md                # ← je bent hier
    └── modules/
        ├── network/             # VNet, Subnet, NSG, Publiek IP, NIC
        │   ├── main.tf
        │   ├── variables.tf
        │   └── outputs.tf
        ├── compute/             # Ubuntu VM + auto-shutdown
        │   ├── main.tf
        │   ├── variables.tf
        │   └── outputs.tf
        └── database/            # MySQL Flexible Server + firewallregels
            ├── main.tf
            ├── variables.tf
            └── outputs.tf
```
