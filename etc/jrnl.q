system "d .jrnl"

/jfnpt - Journal File Path Template
jfpt:""
/jfn - Journal File Name
/jfh - Journal File Hande

/jinit - init / replay journal
jinit:{
    jfn::hsym `$jfpt,string .z.D;
    exists:{0 < @a[hcount; x; {0}]};
    init:{jfn set (); :jfh::hopen jfn;};
    if [not exists jfn;
        0N!"Log: started";
        :init[]];
    0N!"Log: Restarted";
    chunks:-11!(-2;jfn);
    broken:1 < count chunks;
    0N!"Log: No. of valid chunks: ",string first chunks;
    if [broken;
        0N!"Log: Broken. repearing";
        jfn 1: read1 (jfn;0;last chunks);
        .Q.gc[]];
    0N!"Log: Restoring...";
    -11!(first chunks;jfn);
    0N!"Log: Restore finished";
    .Q.gc[];
    :jfh::hopen jfn;
    }

/jupd - update journal
jupd:{jfh enlist x}

/jclr - close and clear journal
jclr:{hclose jfh; hdel jfn}

system "d ."


