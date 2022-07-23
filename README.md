# Adana -  dumb cache / dumb terminal

- use help for help


Static build using MUSL:

```
cargo build --release 
```

should print "statically linked":

```
ldd target/x86_64-unknown-linux-musl/release/adana 

```

### install using cargo

```
cargo install adana --target x86_64-unknown-linux-musl 
```

### logs
RUST_LOG=info adana


### docker

```
docker build -t adana .

docker run -it adana 

```

### Args - override

``` 
# open an in memory db

adana --inmemory

```

```
# override db path & history path + fallback in memory in case of an error (default to false)
# path must exist! file doesn't have to.

adana --dbpath /tmp/mydb.db --historypath /tmp/myhistory.txt --fallback

```