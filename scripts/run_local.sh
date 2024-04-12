#!/bin/bash

script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

trap "cd $script_dir && docker-compose stop" EXIT

pushd $script_dir
docker-compose up -d
popd

RUN_LOCAL=1 cargo lambda watch
