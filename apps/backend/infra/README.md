# verifyOS Backend (AWS via Terraform)

This Terraform module provisions:

- An ECR repository for the backend image
- An App Runner service that runs the container

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

## Env vars

Provide required auth secrets via `var.env_vars`:

- `GOOGLE_CLIENT_ID`
- `GOOGLE_CLIENT_SECRET`
- `GOOGLE_REDIRECT_URL` (e.g. `https://api.verifyos.com/api/v1/auth/google/callback`)
- `FRONTEND_BASE_URL` (e.g. `https://verify-os.vercel.app`)
- `REQUIRE_AUTH` (set to `true` for prod)

Example `terraform.tfvars`:

```hcl
env_vars = {
  REQUIRE_AUTH       = "true"
  FRONTEND_BASE_URL  = "https://verify-os.vercel.app"
  GOOGLE_CLIENT_ID   = "..."
  GOOGLE_CLIENT_SECRET = "..."
  GOOGLE_REDIRECT_URL  = "https://api.verifyos.com/api/v1/auth/google/callback"
}
```
