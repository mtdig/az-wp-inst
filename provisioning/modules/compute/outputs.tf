output "vm_id" {
  description = "Resource-ID van de virtuele machine"
  value       = azurerm_linux_virtual_machine.this.id
}

output "vm_name" {
  description = "Naam van de virtuele machine"
  value       = azurerm_linux_virtual_machine.this.name
}

output "private_ip" {
  description = "Privé IP-adres van de VM"
  value       = azurerm_linux_virtual_machine.this.private_ip_address
}
