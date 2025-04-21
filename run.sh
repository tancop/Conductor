if [ ! -f "~/.steam/steam/.cef-enable-remote-debugging"]; then
    # enable Steam remote debugging
    touch "~/.steam/steam/.cef-enable-remote-debugging"
fi

python ./main.py