# cCheck

A util for scanning minecraft servers. Accepts JSON output from [Masscan](https://github.com/robertdavidgraham/masscan) and pings all services found, checking for minecraft servers. Written in rust, with heavy multithreading.  

```sh
~/c/t/release ❯❯❯ time ./ccheck scan hetzner3.json hetzner3.out3.json -w 1000
✓ Found 3352 good servers out of 5720!
./ccheck scan hetzner3.json hetzner3.out3.json -w 1000  0.24s user 0.55s system 26% cpu 2.965 total
```



Legit, don't use this tool to do anything illegal or unethical. It's for educational purposes exclusively. However, this is a favor I'm asking. All terms of the GPL ultimately govern what you do with this. 

# Usage

Say that your trying to find a server that exists on your local ip range `10.0.0.0/8`, where a player named `CCheck` is playing


```bash
# first, run masscan to find open ports
# banners is needed for ccheck to work
sudo masscan -p 25565 --rate 1000 --banners -oJ scan.json 10.0.0.0/8
# scan for player named ccheck
./c_check scan -w 10 scan.json --include "PlayerName:CCheck" output.json

```

`output.json` will include JSON formatted list of all servers with the player CCheck connected. 

# License & Credits

See [COPYING](COPYING)