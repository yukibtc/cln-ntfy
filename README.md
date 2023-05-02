# Ntfy plugin for CLN

## Clone

```
git clone https://github.com/yukibtc/cln-ntfy.git && cd cln-ntfy
```

## Build

```
make
```

or

```
cargo build --release
```

## Configuration

Edit your `~/.lightning/config` file:

```
plugin=/path/to/cln-ntfy
ntfy-url=https://ntfy.sh              # Mandatory
```

Custom options:

```
ntfy-topic=cln-alerts              
ntfy-username=username                
ntfy-password=password                
ntfy-proxy=socks5h://127.0.0.1:9050   # Needed to use a .onion ntfy url
```

## State

**This project is in an ALPHA state**

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details