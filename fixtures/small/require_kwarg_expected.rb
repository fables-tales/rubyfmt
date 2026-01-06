ThingToCall.call(
  require: "not a regular require",
  other: "value"
)

foo(require: bar)

{
  :require => foo,
  :other => 1
}

"require#{foo}"

x = {require: 1, other: 2}
y = {require:, other:}

def thing(require:)
  require
end
