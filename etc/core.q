system "d .core"

/Last sequence
seq:0j

/Generate EOD?
geneod:1b

/Time to call eod
eodtime:23:00:00v

/Timer period in ms
tp:1000

timerinit:{system "t ",string tp}

system "d ."


