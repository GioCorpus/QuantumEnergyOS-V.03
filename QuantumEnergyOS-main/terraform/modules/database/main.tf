// Database module supporting AWS RDS and Azure PostgreSQL Flexible Server
// Resources are created conditionally based on `var.cloud`.

locals {
	is_aws     = var.cloud == "aws"
	is_azurerm = var.cloud == "azurerm"
}

// --- AWS RDS (Postgres/MySQL) ---
resource "aws_db_instance" "db" {
	count                   = local.is_aws ? 1 : 0
	identifier              = var.identifier
	allocated_storage       = var.allocated_storage
	engine                  = var.engine
	engine_version          = var.engine_version
	instance_class          = var.instance_class
	name                    = var.db_name
	username                = var.username
	password                = var.password
	parameter_group_name    = var.parameter_group
	skip_final_snapshot     = var.skip_final_snapshot
	multi_az                = var.multi_az
	publicly_accessible     = var.publicly_accessible
	vpc_security_group_ids  = var.vpc_security_group_ids
	tags                    = var.tags
}

// --- Azure Database for PostgreSQL Flexible Server ---
resource "azurerm_postgresql_flexible_server" "db" {
	count                      = local.is_azurerm ? 1 : 0
	name                       = var.db_name
	resource_group_name        = var.resource_group_name
	location                   = var.location
	administrator_login        = var.username
	administrator_login_password = var.password
	sku_name                   = var.sku_name
	version                    = var.engine_version
	storage_mb                 = var.allocated_storage * 1024

	high_availability {
		mode = var.high_availability ? "ZoneRedundant" : "Disabled"
	}

	backup {
		backup_retention_days = var.backup_retention_days
	}

	tags = var.tags
}

// --- Outputs ---
output "endpoint" {
	description = "Database endpoint / host"
	value = local.is_aws ? aws_db_instance.db[0].address : (local.is_azurerm ? azurerm_postgresql_flexible_server.db[0].fqdn : null)
}

output "port" {
	description = "Database port"
	value       = var.port
}

output "username" {
	description = "Admin username for the database"
	value       = var.username
}

output "password" {
	description = "Admin password (sensitive)"
	value       = var.password
	sensitive   = true
}

output "identifier" {
	description = "Cloud-specific resource identifier"
	value       = local.is_aws ? aws_db_instance.db[0].id : (local.is_azurerm ? azurerm_postgresql_flexible_server.db[0].id : null)
}

