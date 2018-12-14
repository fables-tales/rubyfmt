#!/bin/bash
time bundle exec ruby  src/rubyfmt.rb fixtures/rspec_core_notifications_actual.rb > out.rb
cat out.rb
