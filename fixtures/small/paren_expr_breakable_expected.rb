# Modifier if that becomes block if due to line length (the original idempotency bug)
{
  descendent_members: (
    if team[:descendant_members]
        .present?
      team[:descendant_members].map { |v| self.trim_person }
    end
  )
}

# Similar case with unless
{
  filtered_items: (
    unless collection[:items]
        .empty?
      collection[:items].select { |item| item.valid? && item.active? }
    end
  )
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
(
  case status
  when :active
    handle_active
  when :pending
    handle_pending
  end
)
