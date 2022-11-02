class DeeplyNestedClass
  def thing
    while foo?
      loop do
        # 118 characters without indentation
        thing
          .golly_gee_what_a_big_surprise
          .that_someone_would_make_a_method_call_that_is_over_the_limit_with_indentation
          .wow
      end
    end
  end
end
