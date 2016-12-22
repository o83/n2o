#!/bin/bash

function book {
printf "Book: \x1b[38;2;74;144;226mAAPL\x1b[0m\n"
printf "id   side time               vol   price venue\n"
printf '==== ==== ================== ===== ===== =====\n'
printf "3    \x1b[38;2;208;002;027mASK  09:05:01:123871012 300   20.30 [1,2]\x1b[0m\n"
#printf "1    \x1b[38;2;208;002;027mASK  09:01:12:192090139 100   20.30 2\x1b[0m\n"
printf "2    \x1b[38;2;208;002;027mASK  09:03:25:716945237 100   20.25 1\x1b[0m\n"
printf "5    \x1b[38;2;126;211;033mBID  09:08:42:134673465 200   20.20 1\x1b[0m\n"
#printf "4    \x1b[38;2;126;211;033mBID  09:06:11:784316783 100   20.15 1\x1b[0m\n"
printf "6    \x1b[38;2;126;211;033mBID  09:09:37:834852874 300   20.15 [1,2]\x1b[0m\n"
}

function ls {
a=0
for f in *.rs **/*.rs **/**/*.rs
do
let "a+=`cat $f | grep $1 | wc -l`"
done
echo $1 $a
}

book ""
ls "struct"
ls "trait"
ls "impl"
ls "enum"
