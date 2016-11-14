collect:{
    0N!`collect;
    system "t 0";
    if [1<count distinct days;
        /REPORT date mismatch;
        'mismatch;
        ];
    getTbls[];
    {@a[{x (exit;0)};x;{}]} peach rdbh;
    rdbh::();
    exit 0;
    }
