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
│   ├── vault.yml                # encrypted secrets (voor deze opdracht niet encrypted)
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

## Beveiliging

De volgende maatregelen worden automatisch toegepast:

| Maatregel | Beschrijving |
|---|---|
| **Wordfence** | Firewall + malware scanner (licentie wordt automatisch geactiveerd via `vault.yml`) |
| **Limit Login Attempts Reloaded** | Brute-force bescherming op wp-login.php |
| **Disable XML-RPC Pingback** | Blokkeert XML-RPC misbruik (DDoS amplificatie, credential brute-force) |
| **fail2ban – wordpress-login** | Bant IP's op serverniveau na 5 mislukte inlogpogingen in 5 min |
| **fail2ban – sshd** | Bant IP's na 3 mislukte SSH pogingen |
| **Apache hardening** | Verbergt serverversie, blokkeert `xmlrpc.php`, beveiligingsheaders (X-Frame-Options, CSP, etc.) |
| **wp-config hardening** | Bestandseditor uitgeschakeld, HTTPS admin afgedwongen, auto security-updates |
| **UFW firewall** | Alleen poort 22, 80, 443 open |
| **SSH hardening** | Wachtwoord-login uitgeschakeld, alleen pubkey authenticatie |
| **Let's Encrypt SSL** | HTTPS met automatische redirect |

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


## Voorbeeld run

```bash
$ export MYSQL_PASS="LetmeIn!"
$ az group list
[
  {
    "id": "/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/NetworkWatcherRG",
    "location": "francecentral",
    "managedBy": null,
    "name": "NetworkWatcherRG",
    "properties": {
      "provisioningState": "Succeeded"
    },
    "tags": null,
    "type": "Microsoft.Resources/resourceGroups"
  }
]
$ make all
terraform -chdir=provisioning apply -var="admin_public_key=$(cat ~/.ssh/id_ed25519_hogent.pub)" -var="mysql_admin_password=LetmeIn!" -auto-approve

Terraform used the selected providers to generate the following execution plan. Resource actions are indicated with the following symbols:
  + create

Terraform will perform the following actions:

  # azurerm_resource_group.main will be created
  + resource "azurerm_resource_group" "main" {
      + id       = (known after apply)
      + location = "francecentral"
      + name     = "SELab-Wordpress"
    }

  # module.compute.azurerm_dev_test_global_vm_shutdown_schedule.this[0] will be created
  + resource "azurerm_dev_test_global_vm_shutdown_schedule" "this" {
      + daily_recurrence_time = "2359"
      + enabled               = true
      + id                    = (known after apply)
      + location              = "francecentral"
      + timezone              = "Romance Standard Time"
      + virtual_machine_id    = (known after apply)

      + notification_settings {
          + email           = "jeroen.vanrenterghem@student.hogent.be"
          + enabled         = true
          + time_in_minutes = 30
        }
    }

  # module.compute.azurerm_linux_virtual_machine.this will be created
  + resource "azurerm_linux_virtual_machine" "this" {
      + admin_username                                         = "osboxes"
      + allow_extension_operations                             = (known after apply)
      + bypass_platform_safety_checks_on_user_schedule_enabled = false
      + computer_name                                          = "azosboxes"
      + disable_password_authentication                        = (known after apply)
      + disk_controller_type                                   = (known after apply)
      + extensions_time_budget                                 = "PT1H30M"
      + id                                                     = (known after apply)
      + location                                               = "francecentral"
      + max_bid_price                                          = -1
      + name                                                   = "azosboxes"
      + network_interface_ids                                  = (known after apply)                                                                                                                      + os_managed_disk_id                                     = (known after apply)                                                                                                                      + patch_assessment_mode                                  = (known after apply)                                                                                                                      + patch_mode                                             = (known after apply)                                                                                                                      + platform_fault_domain                                  = -1
      + priority                                               = "Regular"
      + private_ip_address                                     = (known after apply)
      + private_ip_addresses                                   = (known after apply)                                                                                                                      + provision_vm_agent                                     = (known after apply)                                                                                                                      + public_ip_address                                      = (known after apply)                                                                                                                      + public_ip_addresses                                    = (known after apply)
      + resource_group_name                                    = "SELab-Wordpress"
      + size                                                   = "Standard_B2ats_v2"
      + virtual_machine_id                                     = (known after apply)                                                                                                                      + vm_agent_platform_updates_enabled                      = (known after apply)
      + admin_ssh_key {
          # At least one attribute in this block is (or was) sensitive,
          # so its contents will not be displayed.
        }
      + boot_diagnostics {}

      + os_disk {
          + caching                   = "ReadWrite"                                                                                                                                                           + disk_size_gb              = (known after apply)                                                                                                                                                   + id                        = (known after apply)                                                                                                                                                   + name                      = (known after apply)                                                                                                                                                   + storage_account_type      = "StandardSSD_LRS"                                                                                                                                                     + write_accelerator_enabled = false                                                                                                                                                               }                                                                                                                                                                                                                                                                                                                                                                                                     + source_image_reference {                                                                                                                                                                              + offer     = "ubuntu-22_04-lts"
          + publisher = "canonical"
          + sku       = "server"
          + version   = "latest"
        }

      + termination_notification (known after apply)
    }

  # module.database.azurerm_mysql_flexible_server.this will be created
  + resource "azurerm_mysql_flexible_server" "this" {
      + administrator_login           = "wordpressdb"
      + administrator_password        = (sensitive value)
      + administrator_password_wo     = (write-only attribute)                                                                                                                                            + backup_retention_days         = 7
      + fqdn                          = (known after apply)                                                                                                                                               + geo_redundant_backup_enabled  = false
      + id                            = (known after apply)
      + location                      = "francecentral"                                                                                                                                                   + name                          = "jr-wordpressdb"
      + public_network_access         = (known after apply)
      + public_network_access_enabled = (known after apply)                                                                                                                                               + replica_capacity              = (known after apply)
      + replication_role              = (known after apply)
      + resource_group_name           = "SELab-Wordpress"
      + sku_name                      = "B_Standard_B1ms"
      + version                       = "8.0.21"
      + zone                          = (known after apply)

      + storage {                                                                                                                                                                                             + auto_grow_enabled   = true
          + io_scaling_enabled  = true
          + iops                = (known after apply)
          + log_on_disk_enabled = false                                                                                                                                                                       + size_gb             = 20
        }                                                                                                                                                                                               }

  # module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"] will be created
  + resource "azurerm_mysql_flexible_server_firewall_rule" "rules" {
      + end_ip_address      = "85.201.54.83"
      + id                  = (known after apply)
      + name                = "ClientIPAddress_2026-3-9_8-1-46"
      + resource_group_name = "SELab-Wordpress"
      + server_name         = "jr-wordpressdb"
      + start_ip_address    = "85.201.54.83"
    }

  # module.database.azurerm_mysql_flexible_server_firewall_rule.vm[0] will be created
  + resource "azurerm_mysql_flexible_server_firewall_rule" "vm" {
      + end_ip_address      = (known after apply)
      + id                  = (known after apply)
      + name                = "AllowUbuntuVM"
      + resource_group_name = "SELab-Wordpress"
      + server_name         = "jr-wordpressdb"
      + start_ip_address    = (known after apply)
    }

  # module.network.azurerm_network_interface.this will be created
  + resource "azurerm_network_interface" "this" {
      + accelerated_networking_enabled = true
      + applied_dns_servers            = (known after apply)
      + id                             = (known after apply)
      + internal_domain_name_suffix    = (known after apply)
      + ip_forwarding_enabled          = false
      + location                       = "francecentral"
      + mac_address                    = (known after apply)
      + name                           = "azosboxes911"
      + private_ip_address             = (known after apply)
      + private_ip_addresses           = (known after apply)
      + resource_group_name            = "SELab-Wordpress"
      + virtual_machine_id             = (known after apply)

      + ip_configuration {
          + gateway_load_balancer_frontend_ip_configuration_id = (known after apply)
          + name                                               = "ipconfig1"
          + primary                                            = (known after apply)
          + private_ip_address                                 = (known after apply)                                                                                                                          + private_ip_address_allocation                      = "Dynamic"
          + private_ip_address_version                         = "IPv4"
          + public_ip_address_id                               = (known after apply)
          + subnet_id                                          = (known after apply)
        }
    }

  # module.network.azurerm_network_security_group.this will be created
  + resource "azurerm_network_security_group" "this" {
      + id                  = (known after apply)
      + location            = "francecentral"
      + name                = "azosboxes-nsg"
      + resource_group_name = "SELab-Wordpress"
      + security_rule       = [
          + {
              + access                                     = "Allow"
              + destination_address_prefix                 = "*"
              + destination_address_prefixes               = []
              + destination_application_security_group_ids = []
              + destination_port_range                     = "22"
              + destination_port_ranges                    = []
              + direction                                  = "Inbound"
              + name                                       = "SSH"
              + priority                                   = 300
              + protocol                                   = "Tcp"
              + source_address_prefix                      = "*"
              + source_address_prefixes                    = []
              + source_application_security_group_ids      = []
              + source_port_range                          = "*"
              + source_port_ranges                         = []
                # (1 unchanged attribute hidden)
            },
          + {
              + access                                     = "Allow"
              + destination_address_prefix                 = "*"
              + destination_address_prefixes               = []
              + destination_application_security_group_ids = []
              + destination_port_range                     = "443"
              + destination_port_ranges                    = []
              + direction                                  = "Inbound"
              + name                                       = "HTTPS"
              + priority                                   = 340
              + protocol                                   = "Tcp"
              + source_address_prefix                      = "*"
              + source_address_prefixes                    = []
              + source_application_security_group_ids      = []
              + source_port_range                          = "*"
              + source_port_ranges                         = []
                # (1 unchanged attribute hidden)
            },
          + {
              + access                                     = "Allow"
              + destination_address_prefix                 = "*"
              + destination_address_prefixes               = []
              + destination_application_security_group_ids = []
              + destination_port_range                     = "80"
              + destination_port_ranges                    = []
              + direction                                  = "Inbound"
              + name                                       = "HTTP"
              + priority                                   = 320
              + protocol                                   = "Tcp"
              + source_address_prefix                      = "*"
              + source_address_prefixes                    = []
              + source_application_security_group_ids      = []
              + source_port_range                          = "*"
              + source_port_ranges                         = []
                # (1 unchanged attribute hidden)
            },
        ]
    }

  # module.network.azurerm_public_ip.this will be created
  + resource "azurerm_public_ip" "this" {
      + allocation_method       = "Static"
      + ddos_protection_mode    = "VirtualNetworkInherited"
      + domain_name_label       = "sel-opdracht4-groep99"
      + fqdn                    = (known after apply)
      + id                      = (known after apply)
      + idle_timeout_in_minutes = 4
      + ip_address              = (known after apply)
      + ip_version              = "IPv4"
      + location                = "francecentral"
      + name                    = "azosboxes-ip"
      + resource_group_name     = "SELab-Wordpress"
      + sku                     = "Standard"
      + sku_tier                = "Regional"
    }

  # module.network.azurerm_subnet.this will be created
  + resource "azurerm_subnet" "this" {
      + address_prefixes                              = [
          + "10.0.0.0/24",
        ]
      + default_outbound_access_enabled               = true
      + id                                            = (known after apply)
      + name                                          = "default"
      + private_endpoint_network_policies             = "Disabled"
      + private_link_service_network_policies_enabled = true
      + resource_group_name                           = "SELab-Wordpress"
      + virtual_network_name                          = "azosboxes-vnet"
    }

  # module.network.azurerm_subnet_network_security_group_association.this will be created
  + resource "azurerm_subnet_network_security_group_association" "this" {
      + id                        = (known after apply)
      + network_security_group_id = (known after apply)
      + subnet_id                 = (known after apply)
    }

  # module.network.azurerm_virtual_network.this will be created
  + resource "azurerm_virtual_network" "this" {
      + address_space                  = [
          + "10.0.0.0/16",
        ]
      + dns_servers                    = (known after apply)
      + guid                           = (known after apply)
      + id                             = (known after apply)
      + location                       = "francecentral"
      + name                           = "azosboxes-vnet"
      + private_endpoint_vnet_policies = "Disabled"
      + resource_group_name            = "SELab-Wordpress"
      + subnet                         = (known after apply)
    }

Plan: 12 to add, 0 to change, 0 to destroy.

Changes to Outputs:
  + admin_username    = "osboxes"
  + mysql_admin_login = "wordpressdb"
  + mysql_fqdn        = (known after apply)
  + mysql_server_id   = (known after apply)
  + mysql_server_name = "jr-wordpressdb"
  + public_fqdn       = (known after apply)
  + public_ip_address = (known after apply)
  + vm_id             = (known after apply)
  + vm_name           = "azosboxes"
  + vnet_id           = (known after apply)
azurerm_resource_group.main: Creating...
azurerm_resource_group.main: Still creating... [00m10s elapsed]
azurerm_resource_group.main: Still creating... [00m20s elapsed]
azurerm_resource_group.main: Creation complete after 23s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress]
module.network.azurerm_virtual_network.this: Creating...
module.network.azurerm_public_ip.this: Creating...
module.database.azurerm_mysql_flexible_server.this: Creating...
module.network.azurerm_network_security_group.this: Creating...
module.network.azurerm_network_security_group.this: Creation complete after 3s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.Network/networkSecurityGroups/azosboxes-nsg]
module.network.azurerm_virtual_network.this: Creation complete after 5s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.Network/virtualNetworks/azosboxes-vnet]
module.network.azurerm_subnet.this: Creating...
module.network.azurerm_public_ip.this: Creation complete after 6s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.Network/publicIPAddresses/azosboxes-ip]
module.network.azurerm_subnet.this: Creation complete after 5s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.Network/virtualNetworks/azosboxes-vnet/subnets/default]
module.network.azurerm_subnet_network_security_group_association.this: Creating...
module.network.azurerm_network_interface.this: Creating...
module.database.azurerm_mysql_flexible_server.this: Still creating... [00m10s elapsed]
module.network.azurerm_subnet_network_security_group_association.this: Creation complete after 5s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.Network/virtualNetworks/azosboxes-vnet/subnets/default]
module.network.azurerm_network_interface.this: Creation complete after 7s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.Network/networkInterfaces/azosboxes911]
module.compute.azurerm_linux_virtual_machine.this: Creating...
module.database.azurerm_mysql_flexible_server.this: Still creating... [00m20s elapsed]
module.compute.azurerm_linux_virtual_machine.this: Still creating... [00m10s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [00m30s elapsed]
module.compute.azurerm_linux_virtual_machine.this: Creation complete after 17s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.Compute/virtualMachines/azosboxes]
module.compute.azurerm_dev_test_global_vm_shutdown_schedule.this[0]: Creating...
module.compute.azurerm_dev_test_global_vm_shutdown_schedule.this[0]: Creation complete after 2s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.DevTestLab/schedules/shutdown-computevm-azosboxes]
module.database.azurerm_mysql_flexible_server.this: Still creating... [00m40s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [00m50s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [01m00s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [01m10s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [01m20s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [01m30s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [01m40s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [01m50s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [02m00s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [02m10s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [02m20s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [02m30s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [02m40s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [02m50s elapsed]
module.database.azurerm_mysql_flexible_server.this: Still creating... [03m00s elapsed]
module.database.azurerm_mysql_flexible_server.this: Creation complete after 3m5s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.DBforMySQL/flexibleServers/jr-wordpressdb]
module.database.azurerm_mysql_flexible_server_firewall_rule.vm[0]: Creating...
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Creating...
module.database.azurerm_mysql_flexible_server_firewall_rule.vm[0]: Still creating... [00m10s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [00m10s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.vm[0]: Still creating... [00m20s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [00m20s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.vm[0]: Still creating... [00m30s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [00m30s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.vm[0]: Still creating... [00m40s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [00m40s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.vm[0]: Still creating... [00m50s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [00m50s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [01m00s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.vm[0]: Still creating... [01m00s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.vm[0]: Creation complete after 1m2s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.DBforMySQL/flexibleServers/jr-wordpressdb/firewallRules/AllowUbuntuVM]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [01m10s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [01m20s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [01m30s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [01m40s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [01m50s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Still creating... [02m00s elapsed]
module.database.azurerm_mysql_flexible_server_firewall_rule.rules["ClientIPAddress_2026-3-9_8-1-46"]: Creation complete after 2m4s [id=/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.DBforMySQL/flexibleServers/jr-wordpressdb/firewallRules/ClientIPAddress_2026-3-9_8-1-46]

Apply complete! Resources: 12 added, 0 changed, 0 destroyed.

Outputs:

admin_username = "osboxes"
mysql_admin_login = "wordpressdb"
mysql_fqdn = "jr-wordpressdb.mysql.database.azure.com"
mysql_server_id = "/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.DBforMySQL/flexibleServers/jr-wordpressdb"
mysql_server_name = "jr-wordpressdb"
public_fqdn = "sel-opdracht4-groep99.francecentral.cloudapp.azure.com"
public_ip_address = "20.188.61.11"
vm_id = "/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.Compute/virtualMachines/azosboxes"
vm_name = "azosboxes"
vnet_id = "/subscriptions/725a7bc1-52e3-4084-be64-511580d664c1/resourceGroups/SELab-Wordpress/providers/Microsoft.Network/virtualNetworks/azosboxes-vnet"
──────────────────────────────────────────────
  VM IP         : 20.188.61.11
  Admin user    : osboxes
  MySQL FQDN    : jr-wordpressdb.mysql.database.azure.com
  MySQL admin   : wordpressdb
──────────────────────────────────────────────
cd configuration_management && uv run ansible-playbook playbooks/site.yml \
        -i "20.188.61.11," \
        -u "osboxes" \
        --private-key ~/.ssh/id_ed25519_hogent \
        -e "ansible_host=20.188.61.11" \
        -e "tf_mysql_fqdn=jr-wordpressdb.mysql.database.azure.com" \
        -e "tf_mysql_admin_login=wordpressdb" \
        -e "db_admin_password=LetmeIn!"

PLAY [Volledige WordPress stack] *******************************************************************************************************************************************************************

TASK [Gathering Facts] *****************************************************************************************************************************************************************************
Monday 09 March 2026  20:03:28 +0100 (0:00:00.010)       0:00:00.010 **********
[WARNING]: Host '20.188.61.11' is using the discovered Python interpreter at '/usr/bin/python3.10', but future installation of another Python interpreter could cause a different interpreter to be discovered. See https://docs.ansible.com/ansible-core/2.20/reference_appendices/interpreter_discovery.html for more information.
ok: [20.188.61.11]

TASK [common : SSH beveiligen] *********************************************************************************************************************************************************************
Monday 09 March 2026  20:03:33 +0100 (0:00:04.572)       0:00:04.583 **********
[WARNING]: Module remote_tmp /root/.ansible/tmp did not exist and was created with a mode of 0700, this may cause issues when running as another user. To avoid this, create the remote_tmp dir with the correct permissions manually
changed: [20.188.61.11] => (item={'regexp': '^#?PasswordAuthentication', 'line': 'PasswordAuthentication no'})
changed: [20.188.61.11] => (item={'regexp': '^#?PubkeyAuthentication', 'line': 'PubkeyAuthentication yes'})
changed: [20.188.61.11] => (item={'regexp': '^#?PermitRootLogin', 'line': 'PermitRootLogin prohibit-password'})
changed: [20.188.61.11] => (item={'regexp': '^#?ChallengeResponseAuthentication', 'line': 'ChallengeResponseAuthentication no'})

TASK [common : Pakketten installeren] **************************************************************************************************************************************************************
Monday 09 March 2026  20:03:34 +0100 (0:00:01.309)       0:00:05.892 **********
changed: [20.188.61.11]

TASK [common : Verbinding resetten om nieuwe binaries op te pikken] ********************************************************************************************************************************
Monday 09 March 2026  20:04:49 +0100 (0:01:15.026)       0:01:20.919 **********
[WARNING]: reset_connection task does not support when conditional

TASK [common : Configure UFW] **********************************************************************************************************************************************************************
Monday 09 March 2026  20:04:49 +0100 (0:00:00.049)       0:01:20.968 **********
changed: [20.188.61.11]

TASK [common : fail2ban configureren] **************************************************************************************************************************************************************
Monday 09 March 2026  20:04:52 +0100 (0:00:03.042)       0:01:24.010 **********
changed: [20.188.61.11]

TASK [common : fail2ban WordPress login filter installeren] ****************************************************************************************************************************************
Monday 09 March 2026  20:04:54 +0100 (0:00:01.394)       0:01:25.405 **********
changed: [20.188.61.11]

TASK [common : fail2ban inschakelen] ***************************************************************************************************************************************************************
Monday 09 March 2026  20:04:55 +0100 (0:00:01.080)       0:01:26.485 **********
changed: [20.188.61.11]

TASK [common : neofetch toevoegen aan root bashrc] *************************************************************************************************************************************************
Monday 09 March 2026  20:04:56 +0100 (0:00:01.402)       0:01:27.888 **********
changed: [20.188.61.11]

TASK [common : neofetch toevoegen aan ansible gebruiker bashrc] ************************************************************************************************************************************
Monday 09 March 2026  20:04:57 +0100 (0:00:00.366)       0:01:28.254 **********
changed: [20.188.61.11]

TASK [common : Sudo groep toestaan om sudo te gebruiken zonder wachtwoord] *************************************************************************************************************************
Monday 09 March 2026  20:04:57 +0100 (0:00:00.305)       0:01:28.560 **********
changed: [20.188.61.11]

TASK [mysql_client : Installeer MySQL client en PyMySQL] *******************************************************************************************************************************************
Monday 09 March 2026  20:04:57 +0100 (0:00:00.338)       0:01:28.899 **********
changed: [20.188.61.11]

TASK [mysql_client : Wacht tot Azure MySQL bereikbaar is] ******************************************************************************************************************************************
Monday 09 March 2026  20:05:04 +0100 (0:00:06.525)       0:01:35.424 **********
ok: [20.188.61.11]

TASK [mysql_client : Maak WordPress databank aan op Azure MySQL] ***********************************************************************************************************************************
Monday 09 March 2026  20:05:04 +0100 (0:00:00.504)       0:01:35.929 **********
[WARNING]: Deprecation warnings can be disabled by setting `deprecation_warnings=False` in ansible.cfg.
[DEPRECATION WARNING]: Importing 'to_native' from 'ansible.module_utils._text' is deprecated. This feature will be removed from ansible-core version 2.24. Use ansible.module_utils.common.text.converters instead.
changed: [20.188.61.11]

TASK [mysql_client : Maak WordPress DB gebruiker aan op Azure MySQL] *******************************************************************************************************************************
Monday 09 March 2026  20:05:05 +0100 (0:00:00.552)       0:01:36.481 **********
changed: [20.188.61.11]

TASK [wordpress : Apache en PHP installeren] *******************************************************************************************************************************************************
Monday 09 March 2026  20:05:05 +0100 (0:00:00.610)       0:01:37.092 **********
changed: [20.188.61.11]

TASK [wordpress : Apache modules inschakelen] ******************************************************************************************************************************************************
Monday 09 March 2026  20:05:43 +0100 (0:00:37.857)       0:02:14.950 **********
changed: [20.188.61.11] => (item=rewrite)
changed: [20.188.61.11] => (item=ssl)

TASK [wordpress : WordPress downloaden] ************************************************************************************************************************************************************
Monday 09 March 2026  20:05:45 +0100 (0:00:01.279)       0:02:16.229 **********
changed: [20.188.61.11]

TASK [wordpress : WordPress uitpakken] *************************************************************************************************************************************************************
Monday 09 March 2026  20:05:47 +0100 (0:00:02.161)       0:02:18.391 **********
changed: [20.188.61.11]

TASK [wordpress : wp-config.php aanmaken] **********************************************************************************************************************************************************
Monday 09 March 2026  20:05:49 +0100 (0:00:02.671)       0:02:21.064 **********
changed: [20.188.61.11]

TASK [wordpress : Apache vhost aanmaken] ***********************************************************************************************************************************************************
Monday 09 March 2026  20:05:51 +0100 (0:00:01.167)       0:02:22.232 **********
changed: [20.188.61.11]

TASK [wordpress : WordPress site inschakelen] ******************************************************************************************************************************************************
Monday 09 March 2026  20:05:52 +0100 (0:00:01.064)       0:02:23.297 **********
changed: [20.188.61.11]

TASK [wordpress : Standaard site uitschakelen] *****************************************************************************************************************************************************
Monday 09 March 2026  20:05:52 +0100 (0:00:00.361)       0:02:23.659 **********
changed: [20.188.61.11]

TASK [wordpress : Handlers nu uitvoeren (Apache moet draaien voor certbot)] ************************************************************************************************************************
Monday 09 March 2026  20:05:52 +0100 (0:00:00.313)       0:02:23.973 **********

RUNNING HANDLER [common : SSH herstarten] **********************************************************************************************************************************************************
Monday 09 March 2026  20:05:52 +0100 (0:00:00.011)       0:02:23.984 **********
changed: [20.188.61.11]

RUNNING HANDLER [common : fail2ban herstarten] *****************************************************************************************************************************************************
Monday 09 March 2026  20:05:53 +0100 (0:00:00.469)       0:02:24.454 **********
changed: [20.188.61.11]

RUNNING HANDLER [wordpress : Apache herstarten] ****************************************************************************************************************************************************
Monday 09 March 2026  20:05:53 +0100 (0:00:00.452)       0:02:24.906 **********
changed: [20.188.61.11]

TASK [wordpress : Let's Encrypt certificaat aanvragen via certbot] *********************************************************************************************************************************
Monday 09 March 2026  20:05:54 +0100 (0:00:00.549)       0:02:25.456 **********
changed: [20.188.61.11]

TASK [wordpress : WP-CLI downloaden] ***************************************************************************************************************************************************************
Monday 09 March 2026  20:06:11 +0100 (0:00:17.069)       0:02:42.525 **********
changed: [20.188.61.11]

TASK [wordpress : Controleer of WordPress al geïnstalleerd is] *************************************************************************************************************************************
Monday 09 March 2026  20:06:12 +0100 (0:00:00.844)       0:02:43.370 **********
[WARNING]: Unable to use '/var/www/.ansible/tmp' as temporary directory, falling back to system default: [Errno 13] Permission denied: '/var/www/.ansible'

Unable to use '/var/www/.ansible/tmp' as temporary directory, falling back to system default.

<<< caused by >>>

[Errno 13] Permission denied: '/var/www/.ansible'

ok: [20.188.61.11]

TASK [wordpress : WordPress installeren via WP-CLI] ************************************************************************************************************************************************
Monday 09 March 2026  20:06:13 +0100 (0:00:00.846)       0:02:44.217 **********
changed: [20.188.61.11]

TASK [wordpress : WordPress taal instellen] ********************************************************************************************************************************************************
Monday 09 March 2026  20:06:17 +0100 (0:00:04.014)       0:02:48.231 **********
changed: [20.188.61.11]

TASK [wordpress : Understrap theme installeren en activeren] ***************************************************************************************************************************************
Monday 09 March 2026  20:06:19 +0100 (0:00:02.415)       0:02:50.646 **********
changed: [20.188.61.11]

TASK [wordpress : Beveiligingsplugins installeren en activeren] ************************************************************************************************************************************
Monday 09 March 2026  20:06:23 +0100 (0:00:04.046)       0:02:54.693 **********
changed: [20.188.61.11] => (item=wordfence)
changed: [20.188.61.11] => (item=limit-login-attempts-reloaded)
changed: [20.188.61.11] => (item=disable-xml-rpc-pingback)

TASK [wordpress : Apache headers module inschakelen] ***********************************************************************************************************************************************
Monday 09 March 2026  20:06:44 +0100 (0:00:21.325)       0:03:16.018 **********
changed: [20.188.61.11]

TASK [wordpress : Apache beveiligingsconfiguratie aanmaken] ****************************************************************************************************************************************
Monday 09 March 2026  20:06:45 +0100 (0:00:00.460)       0:03:16.479 **********
changed: [20.188.61.11]

TASK [wordpress : Apache beveiligingsconfiguratie inschakelen] *************************************************************************************************************************************
Monday 09 March 2026  20:06:46 +0100 (0:00:01.096)       0:03:17.575 **********
changed: [20.188.61.11]

TASK [wordpress : wp alias toevoegen aan ansible gebruiker bashrc] *********************************************************************************************************************************
Monday 09 March 2026  20:06:46 +0100 (0:00:00.325)       0:03:17.901 **********
changed: [20.188.61.11]

TASK [wordpress : Ansible gebruiker toevoegen aan www-data groep] **********************************************************************************************************************************
Monday 09 March 2026  20:06:47 +0100 (0:00:00.324)       0:03:18.225 **********
changed: [20.188.61.11]

RUNNING HANDLER [wordpress : Apache herstarten] ****************************************************************************************************************************************************
Monday 09 March 2026  20:06:47 +0100 (0:00:00.555)       0:03:18.781 **********
changed: [20.188.61.11]

TASK [Toon verbindingsinformatie] ******************************************************************************************************************************************************************
Monday 09 March 2026  20:06:48 +0100 (0:00:00.598)       0:03:19.379 **********
ok: [20.188.61.11] =>
    msg:
    - ==========================================
    - 'WordPress is beschikbaar op:'
    - https://sel-opdracht4.groep99.be
    - ''
    - 'MySQL: jr-wordpressdb.mysql.database.azure.com:3306'
    - ==========================================

PLAY [Lokale SSH config bijwerken] *****************************************************************************************************************************************************************

TASK [Backup maken van SSH config] *****************************************************************************************************************************************************************
Monday 09 March 2026  20:06:48 +0100 (0:00:00.050)       0:03:19.430 **********
ok: [localhost]

TASK [SSH config bestand aanmaken als het niet bestaat] ********************************************************************************************************************************************
Monday 09 March 2026  20:06:48 +0100 (0:00:00.215)       0:03:19.645 **********
ok: [localhost]

TASK [Bestaand azosboxes blok verwijderen] *********************************************************************************************************************************************************
Monday 09 March 2026  20:06:48 +0100 (0:00:00.173)       0:03:19.818 **********
changed: [localhost]

TASK [SSH config blok voor azosboxes toevoegen] ****************************************************************************************************************************************************
Monday 09 March 2026  20:06:48 +0100 (0:00:00.226)       0:03:20.044 **********
changed: [localhost]

PLAY RECAP *****************************************************************************************************************************************************************************************
20.188.61.11               : ok=39   changed=35   unreachable=0    failed=0    skipped=0    rescued=0    ignored=0
localhost                  : ok=4    changed=2    unreachable=0    failed=0    skipped=0    rescued=0    ignored=0


TASKS RECAP ****************************************************************************************************************************************************************************************
Monday 09 March 2026  20:06:49 +0100 (0:00:00.162)       0:03:20.207 **********
===============================================================================
common : Pakketten installeren ------------------------------------------------------------------------------------------------------------------------------------------------------------- 75.03s
wordpress : Apache en PHP installeren ------------------------------------------------------------------------------------------------------------------------------------------------------ 37.86s
wordpress : Beveiligingsplugins installeren en activeren ----------------------------------------------------------------------------------------------------------------------------------- 21.33s
wordpress : Let's Encrypt certificaat aanvragen via certbot -------------------------------------------------------------------------------------------------------------------------------- 17.07s
mysql_client : Installeer MySQL client en PyMySQL ------------------------------------------------------------------------------------------------------------------------------------------- 6.53s
Gathering Facts ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 4.57s
wordpress : Understrap theme installeren en activeren --------------------------------------------------------------------------------------------------------------------------------------- 4.05s
wordpress : WordPress installeren via WP-CLI ------------------------------------------------------------------------------------------------------------------------------------------------ 4.01s
common : Configure UFW ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- 3.04s
wordpress : WordPress uitpakken ------------------------------------------------------------------------------------------------------------------------------------------------------------- 2.67s
wordpress : WordPress taal instellen -------------------------------------------------------------------------------------------------------------------------------------------------------- 2.42s
wordpress : WordPress downloaden ------------------------------------------------------------------------------------------------------------------------------------------------------------ 2.16s
common : fail2ban inschakelen --------------------------------------------------------------------------------------------------------------------------------------------------------------- 1.40s
common : fail2ban configureren -------------------------------------------------------------------------------------------------------------------------------------------------------------- 1.39s
common : SSH beveiligen --------------------------------------------------------------------------------------------------------------------------------------------------------------------- 1.31s
wordpress : Apache modules inschakelen ------------------------------------------------------------------------------------------------------------------------------------------------------ 1.28s
wordpress : wp-config.php aanmaken ---------------------------------------------------------------------------------------------------------------------------------------------------------- 1.17s
wordpress : Apache herstarten --------------------------------------------------------------------------------------------------------------------------------------------------------------- 1.15s
wordpress : Apache beveiligingsconfiguratie aanmaken ---------------------------------------------------------------------------------------------------------------------------------------- 1.10s
common : fail2ban WordPress login filter installeren ---------------------------------------------------------------------------------------------------------------------------------------- 1.08s

PLAYBOOK RECAP *************************************************************************************************************************************************************************************
Playbook run took 0 days, 0 hours, 3 minutes, 20 seconds

$
```