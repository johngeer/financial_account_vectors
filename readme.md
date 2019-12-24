# Financial Account Vectors

## Why

To help identify fraud, trends, or economic opportunities, it can be valuable to know how money is flowing through a financial system. 

To do this, we need a way to collect information about how accounts are used. If we can use this information quickly enough, we can notice when money is flowing towards or away from a certain account type.

![Example network (the internet)](img/Internet_map_1024.jpg)

<!-- https://en.wikipedia.org/wiki/Network_topology#/media/File:Internet_map_1024.jpg -->

## What

We can use machine learning on past transactions to understand how an account is used (expressed as a dense vector for each account). This allows us to see how similar accounts are, identify clusters of accounts that are used in a related way (even if they don't transact with each other), and use vector math to quickly spot larger trends.

This repository does the data wrangling necessary to do that. Right now it focuses on the transactions in the bitcoin blockchain. Because this blockchain is public and shows real transactions, it provides a great test dataset. However, this technique can apply to any set of transactions.

Because the most valuable collections of transactions are big, this is designed for speed. It uses the rust programming language, focuses on local data movement (to reduce latency), and uses efficient data structures (RocksDB key value store).

## Usage

This code is written to run on AWS spot instances with EBS volumes for storage. To help set up new instances, it includes some make commands to manage storage, set up the bitcoin node, parse the blockchain, and set up the data structures for machine learning.

### Set up The Instance

1. Start a spot instance with a good amount of memory (> 2 GB)
2. Create volumes if needed
    * One for the Blockchain (`/dev/xvdba`)
    * One for the parsed CSV from the blockchain (`dev/xvdbb`)
3. Set up the instance:

```
sudo make install_docker    # Set up the instance
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
