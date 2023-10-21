#!/usr/bin/env bash

TAG=$(cat version | xargs)
REGISTRY_TAG="ragibkl/api-gibtalk:$TAG"
docker push $REGISTRY_TAG
