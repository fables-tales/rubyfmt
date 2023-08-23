sig do
  params(
    country_names_by_locale: T::Hash[
      String,
      # Comment before
      T::Array[Some::Really::Long::ClassName::ToMakeThis::PassTheMaxLineLength]
      # Comment after
    ]
  )
    .void
end
def foo
end

Array[
  @root_fragment,
  @lemon_tea_fragment,
  @green_tea_fragment,
  @cake_fragment
].sort_by(&:name)
