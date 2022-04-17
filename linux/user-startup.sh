#!/bin/sh
#
# User script for MiSTer OJD Server
#

start() {
        printf "Starting OJD Server: "
        /media/fat/ojd_server/mister_ojd_server
}

stop() {
        printf "Stopping OJD Server: "
        kill -HUP `pidof mister_ojd_server`
}

restart() {
    stop
    start
}


case "$1" in
  start)
        start
        ;;
  stop)
        stop
        ;;
  restart)
        restart
        ;;
  *)
        echo "Usage: $0 {start|stop|restart}"
        exit 1
esac