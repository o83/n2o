system "d .cmdline"

valInt:{valf:{if [null[x]|not[count x]|0=x;'ERR]};@[valf;x;{'"Bad integer param"}];x}
valPathRW:{tstf:` sv x,`testfile;valf:{x set (); hdel x};@[valf;tstf;{'"Bad path param"}];x}
valFileRW:{tstf:`string[x],"testfile";valf:{x set (); hdel x};@[valf;tstf;{'"Bad file param"}];x}

system "d ."


