#!/usr/bin/bash

cd media/tawasol-symbols

folders=$(ls)

for folder in $folders; do
    # echo $folder;
 
    files=$(ls $folder/preview-* 2> /dev/null)
    for file in $files; do
        # echo $file
        renamed=$(echo $file | sed "s/preview/$folder/")
        # echo $renamed
        mv $file $renamed
    done
done
