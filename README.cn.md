## 测试
使用[RSpec](https://rspec.info/)工具进行测试命令行输入输出

这是个Ruby的工具
```bash
# 1.首先安装ruby
$ brew install ruby

# 2.安装RSpec的gem包
$ gem install rspec

# 3.根目录下添加 Gemfile
$ touch Gemfile

# 4.在Gemfile里面添加如下代码
source 'https://rubygems.org'
gem 'github-pages', group: :jekyll_plugins
gem 'webrick'
gem "jekyll-theme-minimal"
gem "rspec"

# 5.执行安装Gemfile里面的依赖
$ bundle install

# 6.在根目录下添加测试文件(spec/**/*_spec.rb)

# 7.执行测试(Default: Run all spec files (i.e., those matching spec/**/*_spec.rb))
$ bundle exec rspec
```

