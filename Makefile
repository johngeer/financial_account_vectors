NODE_NAME=bitcoind-node

upload_btc_node:
	rsync -avz --progress --exclude "\.*" . btc_node:~/btc/
	# rsync -avz -e "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null" --progress . btc_node:/btc/

install_docker:
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

mount_drive:
	# mkfs -t ext4 xvdba # to format the drive
	mount /dev/xvdba ~/btc/bitcoind-data/

btc_start:
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
btc_inspect:
	docker logs -f --tail 10 $(NODE_NAME)
