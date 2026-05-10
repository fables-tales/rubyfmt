a(1)
other_cool_method((a + b).round(4))

# rubocop:disable PrisonGuard/PrivateModule
(foo(
  foo
))
  .flatten # rubocop:enable PrisonGuard/PrivateModule

# rubocop:disable Style/Stuff
(MyModel::InSomeNamespace
  .load_one(
    # rubocop:enable Style/Stuff
    {name: "name"}
  )
  &.rules)
  .freeze

beekeep((the_bees unless not_the_bees!))
