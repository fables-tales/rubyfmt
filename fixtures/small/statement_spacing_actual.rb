# A block call followed by another statement gets a blank line.
a do
end
b

# A do/end annotation stays attached to its definition.
sig do
end
def annotated
end

# A brace annotation also stays attached.
sig { void }
def brace_annotated
end

# An annotation followed by a non-definition is separated.
sig do
end
c

# Comments keep their own spacing rather than getting an injected blank line.
a do
end
# This comment stays attached to the block.
b

# An assignment whose value ends in `end` is also treated as block-like.
X = case
when true
  1
else
  2
end
Y = 1

# A nested block inside a method body is separated from the next statement.
def nested
  a do
  end
  b
end

# Endless methods do not get a blank line after them.
def endless = 1
def after_endless
end
