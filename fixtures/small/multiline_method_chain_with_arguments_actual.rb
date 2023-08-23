# From https://github.com/penelopezone/rubyfmt/issues/307
def stub_server(path:, body: {})
  stub_request(
    :get,
    "https://example.com#{path}"
  ).to_return(body: body.to_json)
end

{
  "original_fields" => foo,
  "alternative_fields" => (thing_one(id, api) + thing_two(
    id,
    api
  )).sort
}