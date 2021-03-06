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

The RHD has couple of sub-commands: **write**, **read**, **mod**

### RHD write:
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
### RHD read:

As the name may suggest the read mode allows the read of the database. This is done in couple different ways and flag combinations. 

As your number of targets rises with time, it will be certainly harder and harder for you to remember all of them. Now you don't have to! Simply ask the RHD with `rhd read --list-all` and the database will return every target created. 

If you use the `--list` flag with the `-t/--target` :
 ```
rhd read -t test_target --list
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
rhd read --list --url -t test_target
```

Finally to see only the url with appropriate status code you can 
```
rhd read -t test_target --status-code 1 --list 
```
in this particular instance the `--status-code` is used as a enabling flag not an actual filter so no matter what you put after the flag the result will be the same 
___________________
___________________

### RHD mod:

RHD mod - allows you to modify the already existing entries in the database. 

**Delete**
One of the most useful function the **mod** allows is to permanently delete entries or whole targets
```
rhd mod -t test_target --status-code 404 -h test_host --delete
```

The scope is dictated with the filters. Testing the scope with **read** can be useful way of checking/preparing the right combination of filters before you accidentally delete entries. 

The demonstrated code used above would delete any entry that has status code of *404* and host set to *test_host*. 

If you tried the **read** functionality, you certainly noticed the `id` column. The **ID** value can be used for the entry you wish to modify without specifying whole bunch of redundant arguments. 

One more thing in regards to the **delete** functionality. If you wish to delete whole target. You can do so with: 
```
rhd mod -t target_to_delete --delete
```
If you specify only the target name and `delete` flag you can delete the whole target and every entry it contains. As a precaution the RHD will ask you to confirm your choice by typing either **y/Y** or **yes/YES** only after you agreed with the deletion the RHD will proceed. 

__________________

**Path Combination**

To be implemented