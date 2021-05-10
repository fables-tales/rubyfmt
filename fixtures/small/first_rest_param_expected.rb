def spawn(
  *args,
  type: @Actor.class,
  channel: Promises::Channel.new,
  environment: Environment,
  name: nil,
  executor: default_executor,
  link: false,
  monitor: false,
  &body
)
end
