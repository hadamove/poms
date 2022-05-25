#!/bin/bash
SOURCE_DIR=`dirname "$0"`
ELECTRON_DIR=${SOURCE_DIR}/../src-electron

source ${SOURCE_DIR}/build-web.sh &
server_pid=$!

cd ${ELECTRON_DIR}
npm install
npm start

kill $server_pid