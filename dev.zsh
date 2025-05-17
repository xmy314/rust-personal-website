#!/bin/zsh
set -euo pipefail
IFS=$'\n\t'

# hot reloading version.
(trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT; \
 zsh -c 'cd frontend; trunk serve --proxy-backend=http://localhost:8081/api/' & \
 zsh -c 'bacon webserver')
