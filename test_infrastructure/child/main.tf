locals {
  name        = "child"
  parent_name = data.terraform_remote_state.parent.outputs.name
}

data "terraform_remote_state" "parent" {
  backend = "local"
  config = {
    path = "../parent/terraform.tfstate"
  }
}

data "terraform_remote_state" "cyclic_detection" {
  backend = "local"
  config = {
    path = "../cycle/terraform.tfstate"
  }
}

output "name" {
  value = local.name
}

output "parent_name" {
  value = local.parent_name
}

output "cyclic_detection" {
  value = data.terraform_remote_state.cyclic_detection.outputs.name
}
