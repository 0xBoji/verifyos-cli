output "service_url" {
  value       = aws_apprunner_service.backend.service_url
  description = "App Runner service URL."
}

output "api_gateway_url" {
  value       = aws_apigatewayv2_api.backend.api_endpoint
  description = "API Gateway HTTP endpoint."
}

output "ecr_repository_url" {
  value       = aws_ecr_repository.backend.repository_url
  description = "ECR repository URL for the backend image."
}
