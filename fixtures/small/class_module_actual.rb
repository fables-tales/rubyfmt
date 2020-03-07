require 'net/http'
require 'pp'

module Cat
  module Dog
    class HTTPClient
      def initialize(target_url)
        @target_url = target_url
      end

      # @public
      def call
        uri = URI(target_url)
        Net::HTTP.get(uri)
      end

      private

      attr_reader :target_url
    end
  end
end

p Cat::Dog::HTTPClient.new("http://example.com/").call
