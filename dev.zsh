#!/bin/zsh
SESH="w_dev"

tmux has-session -t $SESH 2>/dev/null

if [ $? != 0 ]; then
    tmux new-session -d -s $SESH -n "all" -d "cd frontend; trunk serve --proxy-backend=http://localhost:8081/api/"
    tmux split-window -h -t $SESH:0  "bacon webserver"

    # tmux select-window -t $SESH:all
fi

tmux attach-session -t $SESH
