[
  thing.map do |_class_name|
    # Don't create flags for any limiters that look like they're meant
    # to be abstract.
    foo!
    # Similar to how `rate_limiter_name` is implemented in
    # `AbstractRedisLimiter`, we only get a "short" name if the
    # limiter is one of the known legacy packages.
    #
    # For the purposes of flag generation, don't generate a flag for
    # limiters outside of them. They can (and should) still provide
    # their own flag in `flags.yaml` as for any other type of flag.
    name_arts[0]
  end
]
