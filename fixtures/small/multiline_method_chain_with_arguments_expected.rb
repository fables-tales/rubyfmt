# From https://github.com/penelopezone/rubyfmt/issues/307
def stub_server(path:, body: {})
  stub_request(
    :get,
    "https://example.com#{path}"
  )
    .to_return(body: body.to_json)
end
