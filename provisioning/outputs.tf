# =============================================================================
# Netwerk outputs
# =============================================================================
output "public_ip_address" {
  description = "Publiek IP-adres van de VM"
  value       = module.network.public_ip_address
}

output "public_fqdn" {
  description = "DNS naam van het publiek IP (voor CNAME records)"
  value       = module.network.public_fqdn
}

output "vnet_id" {
  description = "Resource-ID van het virtueel netwerk"
  value       = module.network.vnet_id
}

# =============================================================================
# Compute outputs
# =============================================================================
output "vm_id" {
  description = "Resource-ID van de virtuele machine"
  value       = module.compute.vm_id
}

output "vm_name" {
  description = "Naam van de virtuele machine"
  value       = module.compute.vm_name
}

output "admin_username" {
  description = "SSH admin gebruikersnaam op de VM"
  value       = var.admin_username
}

# =============================================================================
# Databank outputs
# =============================================================================
output "mysql_fqdn" {
  description = "Volledig gekwalificeerde domeinnaam van de MySQL server"
  value       = module.database.server_fqdn
}

output "mysql_server_id" {
  description = "Resource-ID van de MySQL server"
  value       = module.database.server_id
}

output "mysql_server_name" {
  description = "Naam van de MySQL server"
  value       = module.database.server_name
}

output "mysql_admin_login" {
  description = "MySQL administrator login"
  value       = var.mysql_admin_login
}
