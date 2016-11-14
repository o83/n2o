/RDB common code

system "l net.q"

/Working in slave mode
.net.mode:1

system "l rdb_custom.q"

upd:{.core.seq::first x; updData x}

eod:{0N!(`eod;x); .net.eod x}

.z.ts:tryreconn

/RDB initialization
init:{createShema[];.core.timerinit[];.net.netinit[];}

@a[init;0b;{exit 1}]


