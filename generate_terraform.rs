fn main() {
    // Generar contenido para variables.tf
    let variables_tf = r#"
variable "cluster_name" {
  type        = string
  description = "Nombre del clúster de Kubernetes"
}

variable "location" {
  type        = string
  description = "Ubicación del clúster"
  default     = "East US"
}

variable "node_count" {
  type        = number
  description = "Número de nodos en el clúster"
  default     = 3
}

variable "vm_size" {
  type        = string
  description = "Tamaño de la VM para los nodos"
  default     = "Standard_DS2_v2"
}
"#;

    // Generar contenido para outputs.tf
    let outputs_tf = r#"
output "cluster_id" {
  value = azurerm_kubernetes_cluster.example.id
}

output "kube_config" {
  value     = azurerm_kubernetes_cluster.example.kube_config_raw
  sensitive = true
}

output "cluster_name" {
  value = azurerm_kubernetes_cluster.example.name
}
"#;

    // Generar contenido para main.tf
    let main_tf = r#"
resource "azurerm_resource_group" "example" {
  name     = "rg-${var.cluster_name}"
  location = var.location
}

resource "azurerm_kubernetes_cluster" "example" {
  name                = var.cluster_name
  location            = azurerm_resource_group.example.location
  resource_group_name = azurerm_resource_group.example.name
  dns_prefix          = var.cluster_name

  default_node_pool {
    name       = "default"
    node_count = var.node_count
    vm_size    = var.vm_size
  }

  identity {
    type = "SystemAssigned"
  }
}
"#;

    println!("Contenido para variables.tf:\n{}", variables_tf);
    println!("\nContenido para outputs.tf:\n{}", outputs_tf);
    println!("\nContenido para main.tf:\n{}", main_tf);
}