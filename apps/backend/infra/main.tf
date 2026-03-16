provider "aws" {
  region = var.aws_region
}

resource "aws_ecr_repository" "backend" {
  name = var.ecr_repo_name
}

data "aws_iam_policy_document" "apprunner_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["build.apprunner.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "apprunner_access" {
  name               = "${var.service_name}-apprunner-ecr"
  assume_role_policy = data.aws_iam_policy_document.apprunner_assume.json
}

resource "aws_iam_role_policy_attachment" "apprunner_ecr" {
  role       = aws_iam_role.apprunner_access.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSAppRunnerServicePolicyForECRAccess"
}

resource "aws_apprunner_service" "backend" {
  service_name = var.service_name

  source_configuration {
    auto_deployments_enabled = false

    authentication_configuration {
      access_role_arn = aws_iam_role.apprunner_access.arn
    }

    image_repository {
      image_identifier      = "${aws_ecr_repository.backend.repository_url}:${var.image_tag}"
      image_repository_type = "ECR"

      image_configuration {
        port = tostring(var.container_port)
        runtime_environment_variables = var.env_vars
      }
    }
  }

  instance_configuration {
    cpu    = var.cpu
    memory = var.memory
  }

  health_check_configuration {
    path                = "/healthz"
    protocol            = "HTTP"
    interval            = 10
    timeout             = 5
    healthy_threshold   = 1
    unhealthy_threshold = 5
  }
}

resource "aws_apigatewayv2_api" "backend" {
  name          = "${var.service_name}-api"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_integration" "backend" {
  api_id             = aws_apigatewayv2_api.backend.id
  integration_type   = "HTTP_PROXY"
  integration_method = "ANY"
  integration_uri    = "https://${aws_apprunner_service.backend.service_url}"
}

resource "aws_apigatewayv2_route" "backend" {
  api_id    = aws_apigatewayv2_api.backend.id
  route_key = "$default"
  target    = "integrations/${aws_apigatewayv2_integration.backend.id}"
}

resource "aws_apigatewayv2_stage" "backend" {
  api_id      = aws_apigatewayv2_api.backend.id
  name        = "$default"
  auto_deploy = true
}
