/FE common code

system "l net.q"
system "l jrnl.q"

data:()

.net.getData:{data where x<data[;0]}

updData:{data,a:enlist[x];.net.pub x}

upd:{
    $a[.net.mode;
        /Slave - take the sequence from the message
        .core.seq::first x;
        /Master - increment .core.seq and add it to the message
        x:(.core.seq+a:1;x)];
    /Log to journal
    .jrnl.jupd(`updData;x);
    /Update data and publish to subscribers
    updData x}

eod:{0N!(`eod;x); .net.eod[x]; exit 0}

.z.ts:{tryreconn[]; tryeod[];}

/Load custom functions for FE
system "l fe_custom.q"

/FE initialization
init:{
    .jrnl.jinit[];
    if [count data; .core.seq::last[data][0]];
    .core.timerinit[];
    .net.netinit[];
    }

@a[init;0b;{exit 1}]

