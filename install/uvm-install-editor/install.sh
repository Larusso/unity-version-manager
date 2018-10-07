#!/usr/bin/env bash
set -x

installer=$1
destination=$2
tmpDestination="$destination/tmp"

mkdir -p $tmpDestination
xar -xf "$installer" -C "$tmpDestination"
tar -C "$destination" -zmxf "$tmpDestination/Unity.pkg.tmp/Payload"
mv $destination/Unity/* $destination
rm -fr "$destination/Unity"
rm -fr $tmpDestination
