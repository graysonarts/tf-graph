locals {
  name       = "cycle"
  cycle_name = data.terraform_remote_state.child.outputs.name
}

data "terraform_remote_state" "child" {
  backend = "local"
  config = {
    path = "../child/terraform.tfstate"
  }
}

output "name" {
  value = local.name
}

output "child_name" {
  value = local.cycle_name
}
