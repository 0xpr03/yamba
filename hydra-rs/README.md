
### Manage-rs
 
Demo manager for yamba-daemon in rust

Has basic playback & queue functionality

#### Building

 - Install [rust](https://www.rust-lang.org/tools/install)
 - Install the type of DB system you want to use.
  - Mariadb/Mysql: TODO ~~Install dev libs (`libmariadb-dev` on apt)~~
  - Postgres: TODO ~~Install dev libs~~
  - Local storage: Nothing further required
 - Build manage-rs with the DB you want to use.  
   All of these commands can be run with `--release` flag depending on whether you want a release build or a fast debug build.
  - Mariadb/Mysql: TODO ~~`cargo build --no-default-features --features maria`~~
  - Postgres: TODO ~~`cargo build --no-default-features --features postgres`~~
  - Local: `cargo build`
  
#### Running  
For Linux it is recommended to use the start.sh and add the IP of your yamba-daemon eg `127.0.0.1:1338`.  
If you have the daemon inside a docker conainer and start manager-rs from you host you can find out the IP via
```bash
sudo docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' <container name>
```
Container name usually being `backend`
