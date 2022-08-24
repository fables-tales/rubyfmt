fields.map do |field|
    <<~DOC
      #{field}
    DOC
  end
  .compact
  .sort
