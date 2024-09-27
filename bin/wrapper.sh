#!/bin/bash

unset RUBYLIB GEM_HOME GEM_PATH BUNDLE_BIN_PATH BUNDLE_GEMFILE

cd "$(dirname "$0")"
cd ..

eval "$(mise env -s bash)"

export GEM_HOME="$(dirname "$0")/../.direnv/ruby"
export GEM_HOME=`readlink -f ${GEM_HOME}`
export GEM_PATH=$GEM_HOME

bundle install
exec bundle exec $*
