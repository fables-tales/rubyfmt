module Example
  class Application < Rails::Application
    config.middleware.insert_before(
      ActionDispatch::Cookies,
      SessionCookieUpgrader
    ) if Rails.env.staging? || Rails.env.production?
  end
end

foo(
  a,
  b
) if c

foo(
  a,
  b
) unless c

foo(
  a,
  b
) if some_method(
  x,
  y
)

foo(a, b) if some_method(
  x,
  y
)

# Surrounding blank lines and a leading comment should be preserved when
# the modifier conditional gets converted to block form.

# leading comment
config.middleware.insert_before(
  ActionDispatch::Cookies,
  SessionCookieUpgrader
) if Rails.env.staging? || Rails.env.production?

# Multiple modifier conditionals separated by blank lines.
foo(
  a
) if c

bar(
  b
) unless d

# Comments and blank lines inside the statement body are preserved.
foo(
  a,
  # mid-arg comment
  b
) if cond

foo(
  a,

  b
) if cond

foo(
  a,

  # comment after blank
  b
) if cond

# Trailing comment on the modifier-if line is hoisted as a leading comment.
foo(
  a,
  b
) if cond # trailing

def m
  foo(
    a,
    b
  ) if cond # trailing
end
