output "server_fqdn" {
  description = "Volledig gekwalificeerde domeinnaam van de MySQL server"
  value       = azurerm_mysql_flexible_server.this.fqdn
}

output "server_id" {
  description = "Resource-ID van de MySQL server"
  value       = azurerm_mysql_flexible_server.this.id
}

output "server_name" {
  description = "Naam van de MySQL server"
  value       = azurerm_mysql_flexible_server.this.name
}
