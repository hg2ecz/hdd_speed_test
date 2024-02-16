# HDD random 4k write test with sync
```
$ cargo build --release
$ ./target/release/hdd-speed-test -h

$ ./target/release/hdd-speed-test -s 8192 -r   # first run, create file
$ ./target/release/hdd-speed-test -s 8192 -r   # test

$ rm tesztfile.dat
```

Measured values:
  * 4 TB HDD software raid1: **20.904** msec/4k
  * 480 GB NVMe (SAMSUNG MZVLQ512HALU-000H1) single: **2.411** / 0.696 msec/4k

SSD time is variable. By new SSD (unfragmantable) is faster, later will be slower.
