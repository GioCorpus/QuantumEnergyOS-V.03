output "droplet_id" {
  description = "Droplet resource ID"
  value       = digitalocean_droplet.example.id
}

output "droplet_name" {
  description = "Droplet name"
  value       = digitalocean_droplet.example.name
}

output "ipv4_address" {
  description = "Droplet public IPv4 address"
  value       = digitalocean_droplet.example.ipv4_address
}

output "ipv6_address" {
  description = "Droplet public IPv6 address, if available"
  value       = try(digitalocean_droplet.example.ipv6_address, null)
}

output "status" {
  description = "Droplet provisioning status"
  value       = digitalocean_droplet.example.status
}

output "region" {
  description = "Droplet region slug"
  value       = digitalocean_droplet.example.region
}

output "tags" {
  description = "Droplet tags"
  value       = digitalocean_droplet.example.tags
}
