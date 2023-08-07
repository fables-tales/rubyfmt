sig do
  params(
    country_names_by_locale: T::Hash[
      String,
      # Comment before
      T::Array[Some::Really::Long::ClassName::ToMakeThis::PassTheMaxLineLength]
      # Comment after
    ]
  ).void
end
def foo
end
