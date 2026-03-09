# modules/database – variabelen

variable "resource_group_name" {
  type = string
}
variable "location" {
  type = string
}
variable "server_name" {
  type = string
}
variable "administrator_login" {
  type = string
}
variable "administrator_password" {
  type      = string
  sensitive = true
}
variable "mysql_version" {
  type    = string
  default = "8.0.21"
}
variable "sku_name" {
  type    = string
  default = "B_Standard_B1ms"
}
variable "server_edition" {
  type    = string
  default = "Burstable"
}
variable "storage_size_gb" {
  type    = number
  default = 20
}
variable "storage_iops" {
  type    = number
  default = 360
}
variable "storage_autogrow" {
  type    = bool
  default = true
}
variable "auto_io_scaling" {
  type    = bool
  default = true
}
variable "backup_retention_days" {
  type    = number
  default = 7
}
variable "geo_redundant_backup" {
  type    = bool
  default = false
}
variable "ha_mode" {
  description = "Disabled, ZoneRedundant, or SameZone"
  type        = string
  default     = "Disabled"
}
variable "allow_vm" {
  description = "Firewallregel aanmaken om de VM toe te laten (true als er een VM is)"
  type        = bool
  default     = false
}
variable "vm_public_ip" {
  description = "Publiek IP van de VM – wordt automatisch toegelaten via de firewall"
  type        = string
  default     = ""
}
variable "firewall_rules" {
  type = list(object({
    name             = string
    start_ip_address = string
    end_ip_address   = string
  }))
  default = []
}
variable "tags" {
  type    = map(string)
  default = {}
}
