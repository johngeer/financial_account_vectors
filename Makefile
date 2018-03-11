NODE_NAME=bitcoind-node
PARSER_TAG=jgeer/blockchain_parser:1.22.1
GRAPH_TAG=jgeer/blockchain_graph:1.22.1
BLOCKCHAIN_FOLDER=bitcoind-data
CSV_FOLDER=csv-data

upload:
	# Upload to the btc_node server
	rsync -avz --progress --exclude="\.*" --include=".dockerignore" . btc_node:~/btc/

setup: install_docker
	apt-get install --assume-yes \
		htop \
		make \
		tmux \
		python-pip
	pip install awscli --upgrade

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
	# mkfs -t ext4 /dev/xvdf # to format the drive
	mount /dev/xvdg ~/btc/$(BLOCKCHAIN_FOLDER)/
	mount /dev/xvdf ~/btc/$(CSV_FOLDER)/

cleanup:
	docker container prune --force
	docker rmi $(shell docker images -f dangling=true -q)

## Bitcoin node
btc_run:
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

btc_log:
	# Show what bitcoind has been up to
	docker logs -f --tail 10 $(NODE_NAME)

## Parse bitcoin blockchain
parser_run:
	# Build the docker image (copies code into it)
	docker build -t $(PARSER_TAG) .
	docker run -d -t \
		--name=parser \
		--mount type=bind,source="$(shell pwd)"/$(BLOCKCHAIN_FOLDER)/.bitcoin/blocks/,target=/home/bitcoin \
		--mount type=bind,source="$(shell pwd)"/$(CSV_FOLDER),target=/home/csv-data \
		$(PARSER_TAG)

parser_upload:
	# Upload the parsed CSVs to S3
	gzip $(CSV_FOLDER)/*.csv
	aws s3 cp $(CSV_FOLDER) s3://blockchain-data-1e42/

parser_log:
	# Show what the parser is up to
	docker logs -f --tail 10 parser

## Interatively run

run_int:
	# Run the container interactively, with the present working directory
	# mounted for easier debugging
	# NOTE: You may need to set the environment variables in order to query
	# from the DB
	docker build -t $(GRAPH_TAG) .
	docker run \
		-ti `# run interactively` \
		--volume "$(shell pwd)":/home/ `#attach the present working directory to the instance` \
		--rm `# remove container when done` \
		$(GRAPH_TAG) `# choose the image` \
		/bin/bash # don't auto-run the script
