terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

provider "azurerm" {
  features {}
}

provider "digitalocean" {
  token = var.digitalocean_token
}

# Database Module
module "database" {
  source = "./modules/database"

  cloud                   = var.cloud_provider
  identifier              = var.db_identifier
  engine                  = var.db_engine
  engine_version          = var.db_engine_version
  instance_class          = var.db_instance_class
  allocated_storage       = var.db_allocated_storage
  db_name                 = var.db_name
  username                = var.db_username
  password                = var.db_password
  port                    = var.db_port
  parameter_group         = var.db_parameter_group
  skip_final_snapshot     = var.db_skip_final_snapshot
  multi_az                = var.db_multi_az
  publicly_accessible     = var.db_publicly_accessible
  vpc_security_group_ids  = var.db_vpc_security_group_ids
  tags                    = var.common_tags

  # Azure-specific
  resource_group_name     = var.azure_resource_group_name
  location                = var.azure_location
  sku_name                = var.db_sku_name
  high_availability       = var.db_high_availability
  backup_retention_days   = var.db_backup_retention_days
}

# Kubernetes Module
module "kubernetes" {
  source = "./modules/kubernetes"

  cluster_name = var.k8s_cluster_name
  location     = var.azure_location
  node_count   = var.k8s_node_count
  vm_size      = var.k8s_vm_size
}

# DigitalOcean Droplet Module
module "droplet" {
  source = "./modules/droplet"

  # If you have droplet-specific variables, add them here
  # droplet_name = var.droplet_name
  # region       = var.droplet_region
  # etc.
}

# Outputs
output "database_endpoint" {
  description = "Database endpoint"
  value       = module.database.endpoint
}

output "database_port" {
  description = "Database port"
  value       = module.database.port
}

output "kubernetes_cluster_id" {
  description = "Kubernetes cluster ID"
  value       = module.kubernetes.cluster_id
}

output "kubernetes_cluster_name" {
  description = "Kubernetes cluster name"
  value       = module.kubernetes.cluster_name
}

output "droplet_id" {
  description = "DigitalOcean Droplet ID"
  value       = module.droplet.droplet_id
}

output "droplet_ipv4_address" {
  description = "DigitalOcean Droplet IPv4 address"
  value       = module.droplet.ipv4_address
}
