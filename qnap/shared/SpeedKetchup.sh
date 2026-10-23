#!/bin/sh
CONF=/etc/config/qpkg.conf
QPKG_NAME=SpeedKetchup
QPKG_ROOT=`/sbin/getcfg $QPKG_NAME Install_Path -f ${CONF}`
PIDFILE=/var/run/$QPKG_NAME.pid
export QNAP_QPKG=$QPKG_NAME

case "$1" in
  start)
    ENABLED=$(/sbin/getcfg $QPKG_NAME Enable -u -d FALSE -f $CONF)
    if [ "$ENABLED" != "TRUE" ]; then
        echo "$QPKG_NAME is disabled."
        exit 1
    fi
    if [ -f $PIDFILE ] && kill -0 $(cat $PIDFILE) 2>/dev/null; then
        echo "$QPKG_NAME is already running."
        exit 0
    fi
    set -a
    . $QPKG_ROOT/speedketchup.conf
    set +a
    # keep the App Center link on the configured port
    /sbin/setcfg $QPKG_NAME Web_Port $SPEEDKETCHUP_PORT -f $CONF
    /sbin/setcfg $QPKG_NAME Service_Port $SPEEDKETCHUP_PORT -f $CONF
    export SPEEDKETCHUP_ADDRESS=0.0.0.0
    export SPEEDKETCHUP_FILE=$QPKG_ROOT/data/speedketchup-results.csv
    mkdir -p $QPKG_ROOT/data
    # stdout has a trace of every test, the results go to the csv file
    # and errors still reach the log on stderr
    $QPKG_ROOT/speedketchup > /dev/null 2>> $QPKG_ROOT/data/speedketchup.log &
    echo $! > $PIDFILE
    ;;

  stop)
    if [ -f $PIDFILE ]; then
        kill $(cat $PIDFILE) 2>/dev/null
        rm -f $PIDFILE
    fi
    ;;

  restart)
    $0 stop
    $0 start
    ;;
  remove)
    ;;

  *)
    echo "Usage: $0 {start|stop|restart|remove}"
    exit 1
esac

exit 0
