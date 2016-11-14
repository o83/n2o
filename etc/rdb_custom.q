/RDB custom functions

system "l cmdline.q"

/Parse command line params

usage:{0N!"Usage: QEXEC rdb.q Listen FEAddr";exit 1}

parseParams:{
    .net.listen::.cmdline.valInt "I"$x 0;
    .net.fea::hsym `$x 1;
    }


if [2<>count .z.x; usage[]]
@[parseParams;.z.x;{0N!x;usage[]}]


system "l fix.q"

createShema:{
    c:.fix.names,`FixMsg`LCID`InOut`TickSeq`unparsed;
    t:.fix.types,"*GSJJ";
    t[where t="c"]:"*";
    fixmsgs::flip c!t$\:()
    }

updData:{
    inout:first last x;
    fixdata:last last x;
    fix:fixdata[0];
    lcid:fixdata[1];
    fd:0N_(!)."I=\001"0:fix;
    unDup:{g:group key x; key[g]!{v:(value y)[x];$[1<count x;"," sv v;first v]}[;x] each value g};
    fd:unDup fd;
    unparsed:count key[fd] except .fix.tags;
    fd:.fix.names!(.fix.types)$fd .fix.tags;
    fd[`FixMsg]:fix;
    fd[`LCID]:lcid;
    fd[`InOut]:inout;
    fd[`TickSeq]:.core.seq;
    fd[`unparsed]:unparsed;
    fixmsgs,:fd;
    .net.pub fd
    }

.net.getData:{select from fixmsgs where TickSeq>x}

