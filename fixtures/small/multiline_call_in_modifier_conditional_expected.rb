module Example
  class Application < Rails::Application
    if Rails.env.staging? || Rails.env.production?
      config.middleware.insert_before(
        ActionDispatch::Cookies,
        SessionCookieUpgrader
      )
    end
  end
end

if c
  foo(
    a,
    b
  )
end

unless c
  foo(
    a,
    b
  )
end

if some_method(
    x,
    y
  )
  foo(
    a,
    b
  )
end

if some_method(
    x,
    y
  )
  foo(a, b)
end

# Surrounding blank lines and a leading comment should be preserved when
# the modifier conditional gets converted to block form.

# leading comment
if Rails.env.staging? || Rails.env.production?
  config.middleware.insert_before(
    ActionDispatch::Cookies,
    SessionCookieUpgrader
  )
end

# Multiple modifier conditionals separated by blank lines.
if c
  foo(
    a
  )
end

unless d
  bar(
    b
  )
end

# Comments and blank lines inside the statement body are preserved.
if cond
  foo(
    a,
    # mid-arg comment
    b
  )
end

if cond
  foo(
    a,

    b
  )
end

if cond
  foo(
    a,

    # comment after blank
    b
  )
end

# Trailing comment on the modifier-if line is hoisted as a leading comment.
# trailing
if cond
  foo(
    a,
    b
  )
end

def m
  # trailing
  if cond
    foo(
      a,
      b
    )
  end
end
