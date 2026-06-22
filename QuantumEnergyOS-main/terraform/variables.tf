# Provider variables
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "digitalocean_token" {
  description = "DigitalOcean API token"
  type        = string
  sensitive   = true
}

# Global configuration
variable "cloud_provider" {
  description = "Primary cloud provider: 'aws' or 'azurerm'"
  type        = string
  default     = "aws"
}

variable "common_tags" {
  description = "Common tags for all resources"
  type        = map(string)
  default = {
    Environment = "dev"
    Project     = "QuantumEnergyOS"
  }
}

# Azure variables
variable "azure_resource_group_name" {
  description = "Azure resource group name"
  type        = string
  default     = "rg-quantumenergy"
}

variable "azure_location" {
  description = "Azure location"
  type        = string
  default     = "East US"
}

# Database variables
variable "db_identifier" {
  description = "Database resource identifier"
  type        = string
  default     = null
}

variable "db_engine" {
  description = "Database engine type"
  type        = string
  default     = "postgres"
}

variable "db_engine_version" {
  description = "Database engine version"
  type        = string
  default     = "13"
}

variable "db_instance_class" {
  description = "AWS RDS instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "db_allocated_storage" {
  description = "Database allocated storage in GB"
  type        = number
  default     = 20
}

variable "db_name" {
  description = "Database name"
  type        = string
  default     = "quantumdb"
}

variable "db_username" {
  description = "Database admin username"
  type        = string
  default     = "admin"
}

variable "db_password" {
  description = "Database admin password"
  type        = string
  sensitive   = true
}

variable "db_port" {
  description = "Database port"
  type        = number
  default     = 5432
}

variable "db_parameter_group" {
  description = "Database parameter group name"
  type        = string
  default     = null
}

variable "db_skip_final_snapshot" {
  description = "Skip final database snapshot"
  type        = bool
  default     = true
}

variable "db_multi_az" {
  description = "Enable Multi-AZ deployment"
  type        = bool
  default     = false
}

variable "db_publicly_accessible" {
  description = "Make database publicly accessible"
  type        = bool
  default     = false
}

variable "db_vpc_security_group_ids" {
  description = "VPC security group IDs"
  type        = list(string)
  default     = []
}

variable "db_sku_name" {
  description = "Azure database SKU name"
  type        = string
  default     = "Standard_B1ms"
}

variable "db_high_availability" {
  description = "Enable database high availability"
  type        = bool
  default     = false
}

variable "db_backup_retention_days" {
  description = "Database backup retention days"
  type        = number
  default     = 7
}

# Kubernetes variables
variable "k8s_cluster_name" {
  description = "Kubernetes cluster name"
  type        = string
  default     = "quantum-aks-cluster"
}

variable "k8s_node_count" {
  description = "Number of Kubernetes nodes"
  type        = number
  default     = 3
}

variable "k8s_vm_size" {
  description = "Kubernetes node VM size"
  type        = string
  default     = "Standard_DS2_v2"
}

# DigitalOcean Droplet variables (can be extended)
variable "droplet_name" {
  description = "DigitalOcean droplet name"
  type        = string
  default     = "quantum-droplet"
}

variable "droplet_region" {
  description = "DigitalOcean droplet region"
  type        = string
  default     = "nyc3"
}

variable "droplet_size" {
  description = "DigitalOcean droplet size slug"
  type        = string
  default     = "s-1vcpu-1gb"
}

variable "droplet_image" {
  description = "DigitalOcean droplet image slug"
  type        = string
  default     = "ubuntu-22-04-x64"
}
