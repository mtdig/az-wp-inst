# modules/compute – variabelen

variable "resource_group_name" {
  type = string
}
variable "location" {
  type = string
}
variable "vm_name" {
  type = string
}
variable "computer_name" {
  type = string
}
variable "vm_size" {
  type = string
}
variable "admin_username" {
  type = string
}
variable "admin_public_key" {
  type      = string
  sensitive = true
}
variable "os_disk_type" {
  type    = string
  default = "StandardSSD_LRS"
}
variable "nic_id" {
  description = "Netwerkinterface-ID om te koppelen"
  type        = string
}
variable "auto_shutdown_enabled" {
  type    = bool
  default = true
}
variable "auto_shutdown_time" {
  type    = string
  default = "2359"
}
variable "auto_shutdown_tz" {
  type    = string
  default = "Romance Standard Time"
}
variable "auto_shutdown_email" {
  type    = string
  default = ""
}
variable "tags" {
  type    = map(string)
  default = {}
}
