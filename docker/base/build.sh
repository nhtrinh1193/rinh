#!/bin/sh
set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"

PROXY=""
if [ "$https_proxy" ]; then
	PROXY=" --build-arg https_proxy=$https_proxy --build-arg http_proxy=$http_proxy"
fi

docker build -f $DIR/base.Dockerfile $DIR/../.. --tag libra_base --build-arg GIT_REV="$(git rev-parse HEAD)" --build-arg GIT_UPSTREAM="$(git merge-base HEAD origin/master)" --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" $PROXY
