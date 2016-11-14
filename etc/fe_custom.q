/FE custom functions

system "l cmdline.q"

/Parse command line params

usage:{0N!"QEXEC fe.q Listen LogFilePathTempl"; exit 1}

parseParams:{
    .net.listen::.cmdline.valInt "I"$x 0;
    .jrnl.jfpt::1_string .cmdline.valFileRW hsym `$x 1;
    }

if [2<>count .z.x; usage[]]
@[parseParams;.z.x;{0N!x;usage[]}]


