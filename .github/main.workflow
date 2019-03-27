workflow "Rubyfmt CI" {
  on = "push"
  resolves = ["CI 2.3", "CI 2.5"]
}

action "CI 2.3" {
  uses = "./dockerfiles/2.3"
  secrets = ["GITHUB_TOKEN"]
}

action "CI 2.5" {
  uses = "./dockerfiles/2.5"
  secrets = ["GITHUB_TOKEN"]
}

action "CI 2.6" {
  uses = "./dockerfiles/2.6"
  secrets = ["GITHUB_TOKEN"]
}
