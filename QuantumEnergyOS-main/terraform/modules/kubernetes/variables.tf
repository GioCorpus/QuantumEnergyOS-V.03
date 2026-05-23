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