# Modifier if that becomes block if due to line length (the original idempotency bug)
{
  descendent_members: (team[:descendant_members].map { |v| self.trim_person } if team[:descendant_members]
    .present?)
}

# Similar case with unless
{
  filtered_items: (collection[:items].select { |item| item.valid? && item.active? } unless collection[:items]
    .empty?)
}

# Already block form should stay the same
(
  if condition
    result
  end
)

# Block unless
(
  unless skip?
    do_something
  end
)

# Case expression in parens
(case status
  when :active
    handle_active
  when :pending
    handle_pending
  end)
