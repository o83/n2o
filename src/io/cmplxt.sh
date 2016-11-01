#!/bin/bash

function ls {
a=0
for f in *.rs **/*.rs
do
let "a+=`cat $f | grep $1 | wc -l`"
done
echo $1 $a
}

ls "struct"
ls "trait"
ls "impl"
