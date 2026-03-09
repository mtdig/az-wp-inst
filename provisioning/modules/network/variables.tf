# modules/network – VNet, Subnet, NSG, Publiek IP, NIC

variable "resource_group_name" {
  type = string
}
variable "location" {
  type = string
}
variable "vnet_name" {
  type = string
}
variable "address_space" {
  type = list(string)
}
variable "subnet_name" {
  type = string
}
variable "subnet_prefix" {
  type = string
}
variable "nsg_name" {
  type = string
}
variable "nsg_rules" {
  type = list(object({
    name                       = string
    priority                   = number
    direction                  = string
    access                     = string
    protocol                   = string
    source_port_range          = string
    destination_port_range     = string
    source_address_prefix      = string
    destination_address_prefix = string
  }))
}
variable "public_ip_name" {
  type = string
}
variable "dns_label" {
  description = "DNS label voor het publiek IP (wordt <label>.francecentral.cloudapp.azure.com)"
  type        = string
  default     = ""
}
variable "nic_name" {
  type = string
}
variable "enable_accelerated" {
  type    = bool
  default = true
}
variable "tags" {
  type    = map(string)
  default = {}
}
