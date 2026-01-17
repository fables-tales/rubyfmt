begin
rescue OpenSSL::SSL::SSLError,
       OpenSSL::SSL::SSLError,
       OpenSSL::SSL::SSLError,
       OpenSSL::SSL::SSLError,
       OpenSSL::SSL::SSLError,
       OpenSSL::SSL::SSLError,
       OpenSSL::SSL::SSLError,
       OpenSSL::SSL::SSLError => exception
end

begin
rescue A, B, C => e
end

begin
rescue StandardError => e
end

begin
rescue A, B
end

begin
rescue StandardError
end
