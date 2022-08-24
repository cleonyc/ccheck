# cCheck

A util for scanning minecraft servers. Accepts JSON output from (Masscan)[https://github.com/robertdavidgraham/masscan] and pings all services found, checking for minecraft servers. Written in rust, with heavy multithreading.  
Designed for you to run your own "Coppenhagen" bot, however it's not fully attached to discord. 

Spaghetti code will be fixed "eventually"

Legit, don't use this tool to do anything illegal or unethical. It's for educational purposes exclusively.

# Usage

Say that your trying to find a server that exists on your local ip range `10.0.0.0/8`, where a player named `CCheck` is playing


```bash
# first, run masscan to find open ports
# banners is needed for ccheck to work
sudo masscan -p 25565 --rate 1000 --banners -oJ scan.json 10.0.0.0/8
# un
./c_check scan -r 500 scan.json

```


# Credits 

See `CREDITS`