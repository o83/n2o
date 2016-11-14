/HDB code

system "l cmdline.q"

listen:0
dbpath:`

delay:5000
days:()
rdba:()
rdbh:()

reConnTO:200

.z.pc:{rdbh[where rdbh=x]:-1}

tryreconn:{
    ra:where rdbh=-1;
    rf:{
        @a[{rdbh[x]:hopen (rdba[x];reConnTO); rdbh[x] (`.net.sub;0Wj)};
        x;
        {if [rdbh[x]<>-1;hclose rdbh[x]; rdbh[x]:-1]} x]};
    rf peach ra;
    }

saveTbl:{[t;n](` sv dbpath,(`$string first distinct days),n,`;20;2;6) set .Q.en[dbpath] t}

getTbls:{
    tbls:first[rdbh] "tables `.";
    {t:raze {y string x}[x] peach rdbh; saveTbl[t;x]} each tbls;
    .Q.chk[dbpath];
    }

collect:{
    0N!`collect;
    system "t 0";
    if [1<count distinct days;
        /REPORT date mismatch
        'mismatch
        ];
    getTbls[];
    {@a[{x (exit;0)};x;{}]} peach rdbh;
    rdbh::();
    exit 0;
    }

upd:{}

eod:{
    0N!(`eod;x);
    if [not .z.w in rdbh; :(::)];
    days,:x;
    if [count[days]=count rdbh;
        system "t 0";
        .z.ts:collect;
        system "t ",string delay];
    }

/Parse command line params
usage:{0N!"Usage: QEXEC hdb.q Listen RDBAddrs DBPath";exit 1}

parseParams:{
    listen::.cmdline.valInt "I"$x 0;
    rdba::hsym `$"," vs x 1;
    rdbh::count[rdba]#-1;
    dbpath::.cmdline.valPathRW hsym `$x 2;
    }

if [3<>count .z.x; usage[]]
@a[parseParams;.z.x;{0N!x;usage[]}]

/Load data
system "l ",1_string dbpath
/Start timer
.z.ts:tryreconn
system "t 1000"
/Start networking
system "p ",string listen


