require "octokit"

GITHUB_CLIENT = Octokit::Client.new(
  access_token: ENV.fetch("GITHUB_TOKEN"),
)

def set_build_status(status)
  GITHUB_CLIENT.create_status(
    ENV.fetch("GITHUB_REPOSITORY"),
    ENV.fetch("GITHUB_SHA"),
    status,
    context: ENV.fetch("CI_BOT_NAME", "rubyfmt CI"),
  )
end
