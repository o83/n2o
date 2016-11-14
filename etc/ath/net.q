system "l core.q"

system "d .net"

/Reconnect timeout in ms
reConnTO:200

/CallBack function to retrieve previous data
getData:{()}
/Remote function name to call on update data
updFunc:`upd
/Remote function name to call on EOD
eodFunc:`eod

/Port listen to
listen:0

/FrontEnd Address
fea:`

/Mode to operate: 0 - master, 1 - slave.
mode:0

/FrontEnd handle
feh:-1

/List of subscribed user hanlers
suh:()

swmode:{mode::x; if [not[x]&feh<>-1; hclose feh; feh::-1]}

sub:{suh::suh union .z.w; getData x}

pub:{{.a[{neg[y] (updFunc;x)};(x;y);{}]}[x] peach suh}

.z.pc:{suh::suh except x; if [feh=x; feh::-1]; x}

eod:{{.a[{y ""; y (eodFunc;x)};(x;y);{}]}[x] peach suh}

netinit:{system "p ",string listen}

system "d ."

tryreconn:{
    if [.net.mode & .net.feh=-1;
        @a[{.net.feh::hopen (.net.fea;.net.reConnTO); upd each .net.feh (`.net.sub;.core.seq)};
            0b;
            {if [.net.feh<>-1; hclose .net.feh; .net.feh::-1]}]
        ];
    }

tryeod:{ if [.core.geneod&not[.net.mode]&.core.eodtime="v"$.z.T; eod[.z.D]] }


