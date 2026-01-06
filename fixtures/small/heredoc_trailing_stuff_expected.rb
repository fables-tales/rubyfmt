module Adapter
  class Test
    describe "start does" do
      it "well" do
        Dir.mktmpdir do |root|

          example_files.each { |file|
            something_to_fill_the_block = do_work(file)

            # Need a comment in here to botch things up.
            more_work = do_work

            error_message = <<~EOS
              X with #{more_computation}.
              This is outdated.

              Another line

              #{insert_computed_thing}
            EOS
            call_the_thing(with_an_arg, error_message)
          }
        end
      end

      it "well" do
        Dir.mktmpdir do |root|

          example_files.each { |file|
            something_to_fill_the_block = do_work(file)

            # Need a comment in here to botch things up.
            more_work = do_work

            error_message = <<~EOS
              X with #{more_computation}.
              This is outdated.

              Another line

              #{insert_computed_thing}
            EOS
            more_work
          }
        end
      end
    end
  end
end
