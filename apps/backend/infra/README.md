# verifyOS Backend (AWS via Terraform)

This Terraform module provisions:

- An ECR repository for the backend image
- An App Runner service that runs the container
- An API Gateway HTTP endpoint in the same region

## Prereqs

- AWS CLI configured locally (`aws sts get-caller-identity` should work)
- Terraform >= 1.5
- Docker

## Deploy

From the repo root:

```bash
cd apps/backend/infra
terraform init
terraform apply
```

Copy the ECR repository URL from the output and push the container image:

```bash
aws ecr get-login-password --region ap-southeast-1 | \
  docker login --username AWS --password-stdin <ECR_REPO_URL>

docker build -f apps/backend/Dockerfile -t verifyos-backend:latest .

docker tag verifyos-backend:latest <ECR_REPO_URL>:latest

docker push <ECR_REPO_URL>:latest
```

Re-run `terraform apply` if you change `var.image_tag`.

After apply, note the API Gateway endpoint from the output `api_gateway_url`.

## Env vars

Provide runtime settings via `var.env_vars` (no defaults are set):

- `RATE_LIMIT_PER_MIN` (e.g. `60`)

Example `terraform.tfvars`:

```hcl
env_vars = {
  RATE_LIMIT_PER_MIN = "60"
}
```
