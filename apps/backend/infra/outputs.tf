output "service_url" {
  value       = aws_apprunner_service.backend.service_url
  description = "App Runner service URL."
}

output "ecr_repository_url" {
  value       = aws_ecr_repository.backend.repository_url
  description = "ECR repository URL for the backend image."
}
