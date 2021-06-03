FROM ubuntu:20.04

ENV TZ=Europe/Zurich
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

COPY docker-install.sh /root
RUN /root/docker-install.sh

COPY docker-init.sh /root
RUN /root/docker-init.sh

ENV HOME=/home
