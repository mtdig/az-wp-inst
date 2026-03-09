# =============================================================================
# Globaal
# =============================================================================
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
  description = "Azure regio voor alle resources"
  type        = string
  default     = "francecentral"
}

variable "tags" {
  description = "Tags toegepast op elke resource"
  type        = map(string)
  default     = {}
}

# =============================================================================
# Netwerk
# =============================================================================
variable "vnet_name" {
  description = "Naam van het virtueel netwerk"
  type        = string
  default     = "azosboxes-vnet"
}

variable "address_space" {
  description = "VNet adresruimte"
  type        = list(string)
  default     = ["10.0.0.0/16"]
}

variable "subnet_name" {
  description = "Naam van het subnet"
  type        = string
  default     = "default"
}

variable "subnet_prefix" {
  description = "Subnet adresprefix"
  type        = string
  default     = "10.0.0.0/24"
}

variable "nsg_name" {
  description = "Naam van de netwerkbeveiligingsgroep"
  type        = string
  default     = "azosboxes-nsg"
}

variable "nsg_rules" {
  description = "Lijst van NSG beveiligingsregels"
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
  default = [
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
    }
  ]
}

variable "public_ip_name" {
  description = "Naam van de publieke IP-resource"
  type        = string
  default     = "azosboxes-ip"
}

variable "public_ip_dns_label" {
  description = "DNS label voor het publiek IP (resulteert in <label>.<regio>.cloudapp.azure.com)"
  type        = string
  default     = ""
}

variable "nic_name" {
  description = "Naam van de netwerkinterface"
  type        = string
  default     = "azosboxes911"
}

variable "enable_accelerated_networking" {
  description = "Versneld netwerken inschakelen op de NIC"
  type        = bool
  default     = true
}

# =============================================================================
# Compute
# =============================================================================
variable "vm_name" {
  description = "Naam van de virtuele machine"
  type        = string
  default     = "azosboxes"
}

variable "computer_name" {
  description = "Hostnaam op OS-niveau"
  type        = string
  default     = "azosboxes"
}

variable "vm_size" {
  description = "VM grootte / SKU"
  type        = string
  default     = "Standard_B2ats_v2"
}

variable "admin_username" {
  description = "SSH admin gebruikersnaam"
  type        = string
  default     = "osboxes"
}

variable "admin_public_key" {
  description = "SSH publieke sleutel voor de admin gebruiker"
  type        = string
  sensitive   = true
}

variable "os_disk_type" {
  description = "Opslagaccounttype voor beheerde schijf"
  type        = string
  default     = "StandardSSD_LRS"
}

variable "auto_shutdown_enabled" {
  description = "Nachtelijke auto-shutdown inschakelen"
  type        = bool
  default     = true
}

variable "auto_shutdown_time" {
  description = "Auto-shutdown tijdstip (UU:mm)"
  type        = string
  default     = "2359"
}

variable "auto_shutdown_tz" {
  description = "Tijdzone voor auto-shutdown"
  type        = string
  default     = "Romance Standard Time"
}

variable "auto_shutdown_email" {
  description = "Notificatie e-mail voor auto-shutdown"
  type        = string
  default     = "jeroen.vanrenterghem@student.hogent.be"
}

# =============================================================================
# Databank – MySQL Flexible Server
# =============================================================================
variable "mysql_server_name" {
  description = "Naam van de MySQL flexible server"
  type        = string
  default     = "jr-wordpressdb"
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

variable "mysql_version" {
  description = "MySQL engine versie"
  type        = string
  default     = "8.0.21"
}

variable "mysql_sku_name" {
  description = "MySQL SKU naam"
  type        = string
  default     = "B_Standard_B1ms"
}

variable "mysql_server_edition" {
  description = "MySQL server editie (Burstable, GeneralPurpose, MemoryOptimized)"
  type        = string
  default     = "Burstable"
}

variable "mysql_storage_size_gb" {
  description = "Opslagruimte in GB"
  type        = number
  default     = 20
}

variable "mysql_storage_iops" {
  description = "Opslag IOPS"
  type        = number
  default     = 360
}

variable "mysql_storage_autogrow" {
  description = "Automatisch opslagvergroting inschakelen"
  type        = bool
  default     = true
}

variable "mysql_auto_io_scaling" {
  description = "Automatische IO-schaling inschakelen"
  type        = bool
  default     = true
}

variable "mysql_backup_retention_days" {
  description = "Back-up retentie in dagen"
  type        = number
  default     = 7
}

variable "mysql_geo_redundant_backup" {
  description = "Geo-redundante back-up inschakelen"
  type        = bool
  default     = false
}

variable "mysql_ha_mode" {
  description = "Hoge beschikbaarheidsmodus (Disabled, ZoneRedundant, SameZone)"
  type        = string
  default     = "Disabled"
}

variable "mysql_firewall_rules" {
  description = "MySQL firewallregels"
  type = list(object({
    name             = string
    start_ip_address = string
    end_ip_address   = string
  }))
  default = [
    {
      name             = "ClientIPAddress_2026-3-9_8-1-46"
      start_ip_address = "85.201.54.83"
      end_ip_address   = "85.201.54.83"
    }
  ]
}
