# HDD random 4k write test with sync

**How does it work?**

* create a file with size length
* rewrite random 4 kbyte in file (or readwrite, -r)
* measure the time:  ```time(N * [4kbyte (rd)wr & sync]) / N``` is the time of the rewriting of 4kbyte block

```
$ cargo build --release
$ ./target/release/hdd-speed-test -h

$ ./target/release/hdd-speed-test -s 8192 -r   # first run, create file
$ ./target/release/hdd-speed-test -s 8192 -r   # only test (file already exists)

$ rm testfile.dat                              # at the end of tests, remove manually
```

Measured values:
  * 4 TB HDD (WD40EZRX) software raid1
     * ext4: **23.18** msec/4k (first part of disk) and **48.80** msec/4k (last part of disk)
     * btrfs: **???** msec/4k (first part of disk) and **167.80** msec/4k (last part of disk)
  * 960 GB SATA (KINGSTON SEDC600) software raid 1 (CPU: Xeon(R) Silver 4116)
    * ext4: **0.067** msec / 4k   (-n 10000)
  * 7,68 TB U.2 (KINGSTON SEDC1500M7680G) software raid1  (CPU: Xeon(R) Silver 4116)
    * ext4: **0,051** msec / 4k
    * btrfs: **0,620** msec / 4k (copy on write filesystem)
