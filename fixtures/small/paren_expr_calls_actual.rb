a (1)
other_cool_method (a + b).round(4)

# rubocop:disable PrisonGuard/PrivateModule
(foo(
  foo # rubocop:enable PrisonGuard/PrivateModule
)).flatten

# rubocop:disable Style/Stuff
(MyModel::InSomeNamespace
  .load_one(
    # rubocop:enable Style/Stuff
    {name: "name"}
  )
  &.rules)
  .freeze
