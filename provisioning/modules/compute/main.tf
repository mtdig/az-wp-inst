# modules/compute – Ubuntu 22.04 VM + auto-shutdown schema

resource "azurerm_linux_virtual_machine" "this" {
  name                = var.vm_name
  resource_group_name = var.resource_group_name
  location            = var.location
  size                = var.vm_size
  computer_name       = var.computer_name
  admin_username      = var.admin_username

  network_interface_ids = [var.nic_id]

  admin_ssh_key {
    username   = var.admin_username
    public_key = var.admin_public_key
  }

  os_disk {
    caching              = "ReadWrite"
    storage_account_type = var.os_disk_type
  }

  source_image_reference {
    publisher = "canonical"
    offer     = "ubuntu-22_04-lts"
    sku       = "server"
    version   = "latest"
  }

  identity {
    type = "SystemAssigned"
  }

  boot_diagnostics {}

  tags = var.tags
}

resource "azurerm_dev_test_global_vm_shutdown_schedule" "this" {
  count = var.auto_shutdown_enabled ? 1 : 0

  virtual_machine_id = azurerm_linux_virtual_machine.this.id
  location           = var.location
  enabled            = true

  daily_recurrence_time = var.auto_shutdown_time
  timezone              = var.auto_shutdown_tz

  notification_settings {
    enabled = var.auto_shutdown_email != "" ? true : false
    email   = var.auto_shutdown_email
  }

  tags = var.tags
}
