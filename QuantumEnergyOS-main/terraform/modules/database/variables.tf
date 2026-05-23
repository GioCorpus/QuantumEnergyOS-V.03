variable "cloud" {
  description = "Cloud provider to target: \"aws\" or \"azurerm\""
  type        = string
  default     = "aws"
}

variable "identifier" {
  description = "Optional identifier for the database resource (AWS)"
  type        = string
  default     = null
}

variable "engine" {
  description = "Database engine (e.g., \"postgres\", \"mysql\")"
  type        = string
  default     = "postgres"
}

variable "engine_version" {
  description = "Engine version to use (cloud-specific)"
  type        = string
  default     = "13"
}

variable "instance_class" {
  description = "AWS RDS instance class (e.g., db.t3.micro)"
  type        = string
  default     = "db.t3.micro"
}

variable "allocated_storage" {
  description = "Allocated storage in GB"
  type        = number
  default     = 20
}

variable "db_name" {
  description = "Name of the initial database"
  type        = string
  default     = "appdb"
}

variable "username" {
  description = "Admin username for the database"
  type        = string
  default     = "admin"
}

variable "password" {
  description = "Admin password for the database"
  type        = string
  sensitive   = true
}

variable "port" {
  description = "Database port"
  type        = number
  default     = 5432
}

variable "parameter_group" {
  description = "Optional parameter group name (AWS)"
  type        = string
  default     = null
}

variable "skip_final_snapshot" {
  description = "When true, skip final DB snapshot on deletion (AWS)"
  type        = bool
  default     = true
}

variable "multi_az" {
  description = "Enable Multi-AZ deployment (AWS)"
  type        = bool
  default     = false
}

variable "publicly_accessible" {
  description = "Make the DB publicly accessible (AWS)"
  type        = bool
  default     = false
}

variable "vpc_security_group_ids" {
  description = "Security group IDs to attach to the DB (AWS)"
  type        = list(string)
  default     = []
}

variable "tags" {
  description = "Tags to apply to created resources"
  type        = map(string)
  default     = {}
}

## Azure-specific variables
variable "resource_group_name" {
  description = "Azure resource group name (azurerm)"
  type        = string
  default     = null
}

variable "location" {
  description = "Azure location for resources (azurerm)"
  type        = string
  default     = null
}

variable "sku_name" {
  description = "SKU name for Azure flexible server (e.g., Standard_B1ms)"
  type        = string
  default     = "Standard_B1ms"
}

variable "high_availability" {
  description = "Enable zone-redundant high availability on Azure"
  type        = bool
  default     = false
}

variable "backup_retention_days" {
  description = "Backup retention period (days) for Azure flexible server"
  type        = number
  default     = 7
}
