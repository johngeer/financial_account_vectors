NODE_NAME=bitcoind-node

upload:
	# Upload to the btc_node server
	rsync -avz --progress --exclude "\.*" . btc_node:~/btc/

install_docker:
	# Setup docker, to allow running the other scripts
	apt-get update
	apt-get install --assume-yes \
		apt-transport-https \
		ca-certificates \
		curl \
		software-properties-common
	curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
	add-apt-repository \
	   "deb [arch=amd64] https://download.docker.com/linux/ubuntu \
	   $(shell lsb_release -cs) \
	   stable"
	apt-get update
	apt-get install --assume-yes docker-ce

ebs_mount:
	# Mount an EBS drive for storing the results
	# mkfs -t ext4 xvdba # to format the drive
	mount /dev/xvdba ~/btc/bitcoind-data/

btc_start:
	# Start the bitcoin node, which will download a copy of the blockchain
	docker container rm $(NODE_NAME) # delete a stopped version of this container
	docker run -d \
		--name=$(NODE_NAME) \
		--mount type=bind,source="$(shell pwd)"/bitcoind-data,target=/bitcoin \
		-p 8333:8333 \
		-p 127.0.0.1:8332:8332 \
		kylemanna/bitcoind
	# docker volume create --name=bitcoind-data
	# -v bitcoind-data:/bitcoin --name=bitcoind-node -d \
		#
btc_log:
	# Show what bitcoind has been up to
	docker logs -f --tail 10 $(NODE_NAME)
