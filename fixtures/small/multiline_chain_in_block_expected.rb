sig {
  params(
    route: String
  )
    .void
}
def ajax_get(route)
  super
end

class Foo
  sig {
    override.returns(
      T::Array[T.class_of(Some::Really::Long::Name::ThatshouldprobablybealisedbutisntbecauseThis::IsATestStub)]
    )
  }
  def example = begin
  end
end
