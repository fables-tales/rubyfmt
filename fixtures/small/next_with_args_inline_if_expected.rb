items.map do |config|
  next config if config.is_a?(UrlConfig)
end
