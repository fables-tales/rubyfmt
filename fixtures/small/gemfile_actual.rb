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
