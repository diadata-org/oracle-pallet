#!/usr/bin/env bash

# REPLACE WITH USER / PROJECT
USER=dia
PROJECT=oracle
VERSION=`grep "^version" ./Cargo.toml | egrep -o "([0-9\.]+)"`

# Build the image
echo "Building ${USER}/${PROJECT}:latest docker image, hang on!"
time docker build -f ./Dockerfile --build-arg RUSTC_WRAPPER= --build-arg PROFILE=release -t ${USER}/${PROJECT}:latest .

# Show the list of available images for this repo
echo "Image is ready"
docker images | grep ${PROJECT}

echo -e "\nIf you just built version ${VERSION}, you may want to update your tag:"
echo " $ docker tag ${USER}/${PROJECT}:$VERSION ${USER}/${PROJECT}:${VERSION}"

