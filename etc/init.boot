ring reader [0-16G]
ring writer [0-16G]
cursor 1 writer 1G
split 1 2 50%
split 2 3 50%
split 1 4 50%
cursor 5 reader 1G
split 5 6 50%
split 5 7 overlapped
reactor aux 0 [console,network]
reactor timer_core 1 [timer]
reactor core1 2 [task]
reactor core2 3 [task]
spawn 1 80 AAPL trader1 core1
spawn 2 80 EEM-SPY-GDX trader1 core1
spawn 3 20 AMI trader1 core1
spawn 5 80 GOOG trader2 core2
spawn 4 80 FB-NFLX-AMZN trader2 core2
timer timer1 core1 SPY rule1 t1 notify
list reactors
list rings
list cursors writer
list core1
list timer_core
send 1 "message2"
send 1 "message2"
dump 1 [0-100]
show recv 1