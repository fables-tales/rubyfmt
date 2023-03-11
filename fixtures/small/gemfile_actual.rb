source "https://rubygems.org"
ruby "2.7.3"
gem "nokogiri"

source "https://my_special_hosted_thing.io" do
  gem "my_hosted_gem", '~>1.2.3'
  gem "rake", :require => false
end

group :test do
  gem 'rspec'
end

# Usages of gemfile methods but in call chains

scope :with_ticket_types, -> { group('shops.id').joins('LEFT JOIN foos ON foos.bar_id
= bars.id').select('foos.*, COUNT(bars.id) count') }

