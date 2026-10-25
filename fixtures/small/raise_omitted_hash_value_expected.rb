def boom(cause)
  raise(ArgumentError, "message", cause:)
end

def retry_with(cause)
  raise(RetryError, cause:)
  cleanup
end

raise(Error, cause:) if cause
