# HDD random 4k write test with sync
```
$ cargo build --release
$ ./target/release/hdd-speed-test -h

$ ./target/release/hdd-speed-test -s 8192 -r   # first run, create file
$ ./target/release/hdd-speed-test -s 8192 -r   # test

$ rm tesztfile.dat
```

Measured values:
  * 4 TB HDD software raid1, ext4: **20.904** msec/4k
  * 480 GB NVMe (SAMSUNG MZVLQ512HALU-000H1) single, ext4: **2.411** / 0.696 msec/4k  (slower value is after 15 full write)
  * 960 GB SATA (KINGSTON SEDC600) software raid 1, ext4: 0.280 msec / 4k
  * 7,68 TB U.2 (KINGSTON SEDC1500M7680G) software raid1
    * ext4: 0,051 msec / 4k
    * btrfs: about 0,560 msec / 4k (copy on write filesystem)
