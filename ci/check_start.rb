require "octokit"

client = if ENV["GITHUB_CLIENT_ID"]
  Octokit::Client.new(
    client_id: ENV.fetch("GITHUB_CLIENT_ID"),
    client_secret: ENV.fetch("GITHUB_CLIENT_SECRET"),
  )
else
  Octokit::Client.new(
    access_token: ENV.fetch("GITHUB_TOKEN"),
  )
end


client.create_status(
  ENV.fetch("GITHUB_REPOSITORY"),
  ENV.fetch("GITHUB_SHA"),
  "pending",
  context: "Build has started",
  description: "rubyfmt CI",
)
