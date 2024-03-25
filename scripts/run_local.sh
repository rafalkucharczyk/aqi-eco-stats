#!/bin/bash

script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

trap "cd $script_dir && docker-compose stop" EXIT

pushd $script_dir
docker-compose up -d
popd

cargo lambda watch
