FROM ubuntu:20.04
RUN export DEBIAN_FRONTEND=noninteractive; apt update -y && apt install -y openssl sqlite ca-certificates git python3-pip tzdata
ADD orm/requirements.txt /tmp/requirements.txt
RUN pip3 install -r /tmp/requirements.txt
WORKDIR /oct
ADD oct-api /oct/oct-api
ADD dist /oct/ui/dist
ADD doc /oct/doc
ADD orm /oct/orm
CMD /oct/oct-api
