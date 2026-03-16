variable "aws_region" {
  type        = string
  description = "AWS region to deploy into."
  default     = "ap-southeast-1"
}

variable "service_name" {
  type        = string
  description = "App Runner service name."
  default     = "verifyos-backend"
}

variable "ecr_repo_name" {
  type        = string
  description = "ECR repository name for the backend image."
  default     = "verifyos-backend"
}

variable "image_tag" {
  type        = string
  description = "Container image tag to deploy."
  default     = "latest"
}

variable "container_port" {
  type        = number
  description = "Container port exposed by the backend."
  default     = 7070
}

variable "cpu" {
  type        = string
  description = "App Runner CPU setting."
  default     = "1024"
}

variable "memory" {
  type        = string
  description = "App Runner memory setting."
  default     = "2048"
}

variable "env_vars" {
  type        = map(string)
  description = "Environment variables passed into the backend container."
  default = {
    REQUIRE_AUTH      = "true"
    FRONTEND_BASE_URL = "https://verify-os.vercel.app"
  }
}
