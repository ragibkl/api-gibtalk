#!/usr/bin/env bash

magick mogrify -format png -colorspace sRGB -density 96 *.svg
