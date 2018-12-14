workflow "New workflow" {
  on = "push"
  resolves = ["Rubyfmt CI"]
}

action "Rubyfmt CI" {
  uses = "./"
  secrets = ["GITHUB_TOKEN"]
}
