# modules/database – MySQL Flexible Server + firewallregels

resource "azurerm_mysql_flexible_server" "this" {
  name                = var.server_name
  resource_group_name = var.resource_group_name
  location            = var.location

  administrator_login    = var.administrator_login
  administrator_password = var.administrator_password

  sku_name = var.sku_name
  version  = var.mysql_version

  backup_retention_days        = var.backup_retention_days
  geo_redundant_backup_enabled = var.geo_redundant_backup

  storage {
    size_gb            = var.storage_size_gb
    iops               = var.auto_io_scaling ? null : var.storage_iops
    auto_grow_enabled  = var.storage_autogrow
    io_scaling_enabled = var.auto_io_scaling
  }

  dynamic "high_availability" {
    for_each = var.ha_mode != "Disabled" ? [1] : []
    content {
      mode = var.ha_mode
    }
  }

  tags = var.tags
}

resource "azurerm_mysql_flexible_server_firewall_rule" "rules" {
  for_each = { for rule in var.firewall_rules : rule.name => rule }

  name                = each.value.name
  resource_group_name = var.resource_group_name
  server_name         = azurerm_mysql_flexible_server.this.name
  start_ip_address    = each.value.start_ip_address
  end_ip_address      = each.value.end_ip_address
}

resource "azurerm_mysql_flexible_server_firewall_rule" "vm" {
  count = var.allow_vm ? 1 : 0

  name                = "AllowUbuntuVM"
  resource_group_name = var.resource_group_name
  server_name         = azurerm_mysql_flexible_server.this.name
  start_ip_address    = var.vm_public_ip
  end_ip_address      = var.vm_public_ip
}
