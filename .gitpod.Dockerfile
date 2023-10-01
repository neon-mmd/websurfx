FROM gitpod/workspace-rust

RUN sudo install-packages redis-server nodejs npm liblua5.4-dev liblua5.3-dev liblua5.2-dev liblua5.1-0-dev libluajit-5.1-dev
