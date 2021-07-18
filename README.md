# Rusty Hunter's Database

is an simple database cli client written in Rust. 

## Installation 

### Prerequisites 

    As this is a Postgresql database client, you will need the Postgres itself, there are plenty of tutorials on how to setup Postgres.

### Installation itself is easy as:

    1. Cloning this repo
    2. cd /repo_location/scripts and run the script as root ( this will setup a separate db and db admin)
    3. If you have Rust installed on your system you can simply build it from the source and then moving the binary to your $PATH / **Currently not available** (Alternatively you can download pre-build binaries from the repo.) 
    
    Optional if you have a your postgres custom configured the RHD will create `.rhd_config` where you can specify all the details needed 


## Using the RHD

The main idea behind the RHD is the constant frustration with all of the different files you usually create while doing a recon on your target. That + me wanting to create something in Rust as an exercise lead me to RHD.

The RHD has couple of modes

**RHD write**:
 _____________
 This mode allows you to import the recon data either through piping the `stdout` of other programs directly or specifying already existing file directly. 

```
rhd write -t test_host -f path/to/file
``` 
 This is the easiest way of importing a data to RHD ( you can import files with status code specified after the url e.g "google.com (200)" as the RHD will try and infer the code automatically and store it appropriately )
______________
```
rhd write -t test_host --stdin
``` 
If you want to pipe data from other programs or `cat` the files and import them in that way into the RHD the `--stdin` flag is the one you want to use

If for some reason the RHD is wrongly inferring the host part of the urls or you want to structure the RHD in your way, you can do so by specifying the host flag `--host / -h` which will override the host extraction from the urls and will instead use the host name specified by you instead 
```
rhd write -t test_host --host test_host_name --stdin
```
all urls that will be imported regardless of the url will have the `test_host_name` as their host in RHD. 
____________
_____________
**RHD read**

As the name may suggest the read mode allows the read of the database. This is done in couple different ways and flag combinations. 

As your number of targets rises with time, it will be certainly harder and harder for you to remember all of them. Now you don't have to! Simply ask the RHD with `rhd read --list-all` and the database will return every target created. 

If you use the `--list-all` flag with the `-t/--target` :
 ```
rhd read -t test_target --list-all
 ``` 
 the RHD will return every entry in the specified target registry. 

If you wish to filter the entries displayed - you can do so by using either `--status-code` (where you can specify the status codes you wish to filter the results by ) or `-h /--host` - or you can combine the two! 
```
rhd read -t test_target --status-code 200, 301 -h test_host, test_host2 
```
This would return only the entries that satisfy all of the conditions. 

That's not all! It is certainly nice to be able to look at the result entries but if you wish to utilize the RHD to the fullest you can use `-u/--url` or `--path` respectively. With these flags you can extract only the urls/paths from the database or pipe the directly to another tool and then back to the RHD

For example if we take the previous query 

```
rhd read -t test_target --status-code 200, 301 -h test_host, test_host2 --url
```
and tag the `--url` flag at the end, then instead of getting result with all of the info for the matching query, we will only receive the `url` part of the entries in `stdout` form, which can be then saved to file/piped to another program 

To list every `url` in the particular target we can simply use the `--list-all` with `--url` 
```
rhd read --list-all --url -t test_target
```
