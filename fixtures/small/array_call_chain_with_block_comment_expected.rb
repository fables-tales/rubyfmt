[
  CommonFields::OBVIOUSLY,
  CommonFields::THESE,
  CommonFields::ARE,
  CommonFields::FAKE_RESOURCE.with_wombo(true).with_combo(true).with_explosion(true)
]
  .map do |field|
    # Swap things out, to make sure.
    override || field
  end
