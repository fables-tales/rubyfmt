#!/bin/bash
time bundle exec ruby  run.rb fixtures/rspec_core_notifications_actual.rb > out.rb
cat out.rb
