FROM ubuntu:20.04

ENV TZ=Europe/Zurich
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

COPY init.sh /root
RUN /root/init.sh
ENV PATH="/root/.cargo/bin:$PATH"
