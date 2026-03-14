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
# Resourcegroep
# -----------------------------------------------------------------------------
resource "azurerm_resource_group" "main" {
  name     = var.resource_group_name
  location = var.location
  tags     = var.tags
}

# -----------------------------------------------------------------------------
# Netwerk – VNet, Subnet, NSG, Publiek IP
# -----------------------------------------------------------------------------
module "network" {
  source = "./modules/network"

  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location

  vnet_name          = var.vnet_name
  address_space      = var.address_space
  subnet_name        = var.subnet_name
  subnet_prefix      = var.subnet_prefix
  nsg_name           = var.nsg_name
  nsg_rules          = var.nsg_rules
  public_ip_name     = var.public_ip_name
  dns_label          = var.public_ip_dns_label
  nic_name           = var.nic_name
  enable_accelerated = var.enable_accelerated_networking

  tags = var.tags
}

# -----------------------------------------------------------------------------
# Compute – Ubuntu 22.04 VM + auto-shutdown schema
# -----------------------------------------------------------------------------
module "compute" {
  source = "./modules/compute"

  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location

  vm_name          = var.vm_name
  vm_size          = var.vm_size
  computer_name    = var.computer_name
  admin_username   = var.admin_username
  admin_public_key = var.admin_public_key
  os_disk_type     = var.os_disk_type
  nic_id           = module.network.nic_id

  auto_shutdown_enabled = var.auto_shutdown_enabled
  auto_shutdown_time    = var.auto_shutdown_time
  auto_shutdown_tz      = var.auto_shutdown_tz
  auto_shutdown_email   = var.auto_shutdown_email

  tags = var.tags
}

# -----------------------------------------------------------------------------
# Managed Identity – Contributor rol voor VM (Semaphore/Terraform auth)
# -----------------------------------------------------------------------------
data "azurerm_subscription" "current" {}

resource "azurerm_role_assignment" "vm_contributor" {
  scope                = data.azurerm_subscription.current.id
  role_definition_name = "Contributor"
  principal_id         = module.compute.identity_principal_id
}

# -----------------------------------------------------------------------------
# Databank – MySQL Flexible Server + firewallregels
# -----------------------------------------------------------------------------
module "database" {
  source = "./modules/database"

  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location

  server_name            = var.mysql_server_name
  administrator_login    = var.mysql_admin_login
  administrator_password = var.mysql_admin_password
  mysql_version          = var.mysql_version
  sku_name               = var.mysql_sku_name
  server_edition         = var.mysql_server_edition

  storage_size_gb    = var.mysql_storage_size_gb
  storage_iops       = var.mysql_storage_iops
  storage_autogrow   = var.mysql_storage_autogrow
  auto_io_scaling    = var.mysql_auto_io_scaling

  backup_retention_days = var.mysql_backup_retention_days
  geo_redundant_backup  = var.mysql_geo_redundant_backup
  ha_mode               = var.mysql_ha_mode

  firewall_rules = var.mysql_firewall_rules
  allow_vm       = true
  vm_public_ip   = module.network.public_ip_address

  tags = var.tags
}
