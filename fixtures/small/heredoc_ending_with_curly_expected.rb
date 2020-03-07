raise <<-EOM.gsub(/^\s+\|/, "")
              |#{"*" * 50}
              |:#{key} is not allowed
              |
              |RSpec reserves some hash keys for its own internal use,
              |including :#{key}, which is used on:
              |
              |  #{CallerFilter.first_non_rspec_line}.
              |
              |Here are all of RSpec's reserved hash keys:
              |
              |  #{RESERVED_KEYS.join("\n  ")}
              |#{"*" * 50}
EOM
