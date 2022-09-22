case type
when AttemptType::WellsFargoUsUsdAchCredits
  # pending for wellsACH
  trace_number_response = TraceNumberResponse.pending
else
# Default for all non-supported attempt types
  trace_number_response = TraceNumberResponse.unsupported
end

case type
when Thing
  # Do Nothing
# Just Vibe
else
# Default for all non-supported attempt types
  T.absurd
end
