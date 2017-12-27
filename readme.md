These are some commands and scripts to make getting and parsing the bitcoin blockchain a little easier.

It is written to use with AWS spot instances and EBS volumes for storage.

## Usage

### Setup The Instance

1. Start a spot instance with a good amount of memory (> 2 GB)
2. Create volumes if needed
    * One for the Blockchain (`/dev/xvdba`)
    * One for the parsed CSV from the blockchain (`dev/xvdbb`)
3. Setup the instance:

```
sudo make install_docker    # Setup the instance
sudo make ebs_mount         # Mount EBS volumes
```

The first time you connect the EBS volumes, you may need to format them with a command like `mkfs -t ext4 xvdba`.

### Download The Bitcoin Blockchain

```
sudo make btc_start
```

To see how things are going: `sudo make btc_log`

### Parse the Bitcoin Blockchain

```
sudo make run_parser
```
