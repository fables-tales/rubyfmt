module UI
  module Draw
    def self.text
      run_thing(
        [
          <<~CMD
          print("hello world")
          CMD
        ]
      )
    end
  end
end
