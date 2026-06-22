output "endpoint" {
  description = "Database endpoint / host (AWS address or Azure FQDN)"
  value       = local.is_aws ? aws_db_instance.db[0].address : (local.is_azurerm ? azurerm_postgresql_flexible_server.db[0].fqdn : null)
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
  description = "Cloud-specific resource identifier (AWS DB instance ID or Azure server ID)"
  value       = local.is_aws ? aws_db_instance.db[0].id : (local.is_azurerm ? azurerm_postgresql_flexible_server.db[0].id : null)
}

output "engine" {
  description = "Database engine type"
  value       = var.engine
}

output "engine_version" {
  description = "Database engine version"
  value       = var.engine_version
}

output "database_name" {
  description = "Name of the created database"
  value       = var.db_name
}

output "aws_db_resource_id" {
  description = "AWS RDS DB resource ID (only when using AWS)"
  value       = try(aws_db_instance.db[0].resource_id, null)
}

output "aws_db_arn" {
  description = "AWS RDS DB ARN (only when using AWS)"
  value       = try(aws_db_instance.db[0].arn, null)
}

output "aws_db_endpoint" {
  description = "AWS RDS endpoint (only when using AWS)"
  value       = try(aws_db_instance.db[0].endpoint, null)
}

output "azure_server_id" {
  description = "Azure PostgreSQL Flexible Server ID (only when using Azure)"
  value       = try(azurerm_postgresql_flexible_server.db[0].id, null)
}

output "azure_server_fqdn" {
  description = "Azure PostgreSQL Flexible Server FQDN (only when using Azure)"
  value       = try(azurerm_postgresql_flexible_server.db[0].fqdn, null)
}

output "cloud_provider" {
  description = "Cloud provider in use"
  value       = local.is_aws ? "aws" : (local.is_azurerm ? "azurerm" : "unknown")
}
