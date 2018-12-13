workflow "New workflow" {
  on = "push"
  resolves = ["./ci/Dockerfile"]
}

action "./ci/Dockerfile" {
  uses = "./ci/Dockerfile"
}
