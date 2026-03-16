# =============================================================================
# Semaphore Terraform – student VM provisioning
#
# Vereenvoudigde versie van provisioning/main.tf voor gebruik door de Deployer
# webapp via Semaphore. Alle variabelen komen via TF_VAR_* environment vars.
# Geen role_assignment (student VMs hebben geen MSI nodig).
# Modules worden gedeeld met de lokale provisioning.
# =============================================================================

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 4.0"
    }
  }
}

provider "azurerm" {
  features {}
  resource_provider_registrations = "none"
  resource_providers_to_register  = [
    "Microsoft.Compute",
    "Microsoft.Network",
    "Microsoft.DBforMySQL",
    "Microsoft.Storage",
  ]
  subscription_id = var.subscription_id
}

# -----------------------------------------------------------------------------
# Variabelen (minimale set — rest heeft defaults in de modules)
# -----------------------------------------------------------------------------
variable "subscription_id" {
  description = "Azure abonnements-ID"
  type        = string
}

variable "resource_group_name" {
  description = "Naam van de resourcegroep"
  type        = string
  default     = "SELab-Wordpress"
}

variable "location" {
  description = "Azure regio"
  type        = string
  default     = "francecentral"
}

variable "public_ip_dns_label" {
  description = "DNS label voor publiek IP"
  type        = string
}

variable "mysql_server_name" {
  description = "Naam van de MySQL Flexible Server"
  type        = string
}

variable "mysql_admin_login" {
  description = "MySQL administrator login"
  type        = string
  default     = "wordpressdb"
}

variable "mysql_admin_password" {
  description = "MySQL administrator wachtwoord"
  type        = string
  sensitive   = true
}

variable "admin_public_key" {
  description = "SSH publieke sleutel voor de VM admin"
  type        = string
}

# -----------------------------------------------------------------------------
# Resourcegroep
# -----------------------------------------------------------------------------
resource "azurerm_resource_group" "main" {
  name     = var.resource_group_name
  location = var.location
}

# -----------------------------------------------------------------------------
# Netwerk
# -----------------------------------------------------------------------------
module "network" {
  source = "../../provisioning/modules/network"

  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  dns_label           = var.public_ip_dns_label

  # Defaults uit de module
  vnet_name          = "azosboxes-vnet"
  address_space      = ["10.0.0.0/16"]
  subnet_name        = "default"
  subnet_prefix      = "10.0.0.0/24"
  nsg_name           = "azosboxes-nsg"
  public_ip_name     = "azosboxes-ip"
  nic_name           = "azosboxes-nic"
  enable_accelerated = false

  nsg_rules = [
    {
      name                       = "SSH"
      priority                   = 300
      direction                  = "Inbound"
      access                     = "Allow"
      protocol                   = "Tcp"
      source_port_range          = "*"
      destination_port_range     = "22"
      source_address_prefix      = "*"
      destination_address_prefix = "*"
    },
    {
      name                       = "HTTP"
      priority                   = 320
      direction                  = "Inbound"
      access                     = "Allow"
      protocol                   = "Tcp"
      source_port_range          = "*"
      destination_port_range     = "80"
      source_address_prefix      = "*"
      destination_address_prefix = "*"
    },
    {
      name                       = "HTTPS"
      priority                   = 340
      direction                  = "Inbound"
      access                     = "Allow"
      protocol                   = "Tcp"
      source_port_range          = "*"
      destination_port_range     = "443"
      source_address_prefix      = "*"
      destination_address_prefix = "*"
    },
  ]

  tags = {}
}

# -----------------------------------------------------------------------------
# Compute – VM (geen role_assignment, student VMs hebben geen MSI nodig)
# -----------------------------------------------------------------------------
module "compute" {
  source = "../../provisioning/modules/compute"

  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  admin_public_key    = var.admin_public_key
  nic_id              = module.network.nic_id

  # Defaults
  vm_name       = "azosboxes"
  vm_size       = "Standard_B2s"
  computer_name = "azosboxes"
  admin_username = "osboxes"
  os_disk_type  = "Standard_LRS"

  auto_shutdown_enabled = true
  auto_shutdown_time    = "0000"
  auto_shutdown_tz      = "Romance Standard Time"
  auto_shutdown_email   = ""

  tags = {}
}

# -----------------------------------------------------------------------------
# Database – MySQL Flexible Server
# -----------------------------------------------------------------------------
module "database" {
  source = "../../provisioning/modules/database"

  resource_group_name    = azurerm_resource_group.main.name
  location               = azurerm_resource_group.main.location
  server_name            = var.mysql_server_name
  administrator_login    = var.mysql_admin_login
  administrator_password = var.mysql_admin_password

  # Defaults
  mysql_version      = "8.0.21"
  sku_name           = "B_Standard_B1ms"
  server_edition     = "Burstable"
  storage_size_gb    = 20
  storage_iops       = 360
  storage_autogrow   = true
  auto_io_scaling    = false
  backup_retention_days = 7
  geo_redundant_backup  = false
  ha_mode               = "Disabled"

  firewall_rules = []
  allow_vm       = true
  vm_public_ip   = module.network.public_ip_address

  tags = {}
}

# -----------------------------------------------------------------------------
# Outputs (geparsed door de Deployer orchestrator)
# -----------------------------------------------------------------------------
output "public_ip_address" {
  value = module.network.public_ip_address
}

output "public_fqdn" {
  value = module.network.public_fqdn
}

output "admin_username" {
  value = "osboxes"
}

output "mysql_fqdn" {
  value = module.database.server_fqdn
}

output "mysql_admin_login" {
  value = var.mysql_admin_login
}
