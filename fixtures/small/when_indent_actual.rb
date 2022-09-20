case type
when AttemptType::WellsFargoUsUsdAchCredits
  # pending for wellsACH
  trace_number_response = TraceNumberResponse.pending
else
# Default for all non-supported attempt types
  trace_number_response = TraceNumberResponse.unsupported
end
