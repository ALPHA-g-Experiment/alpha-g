# `alpha-g-copy`

Make local copies of the MIDAS files from specific runs of the ALPHA-g
experiment. Run the `alpha-g-copy --help` command to make sure you have 
installed the `alpha-g-analysis` package and print help information.

## Requirements

This executable will only run properly on Unix operating systems with the
`rsync` command available. Additionally, you will need a `user` account in
any remote `source` you select (see `alpha-g-copy --help`). 

## Passwordless authentication

Authentication to the server is done by the `rsync` command, hence regular
public key authentication will work as usual. Lxplus is currently the only host 
that doesn't allow public key authentication; in this case you need to obtain a
Kerberos ticket with:

```
kinit -f me@CERN.CH
```

and modify your `~/.ssh/config` file as:

```
Host lxplus*
    Hostname lxplus.cern.ch
    User me
    GSSAPIAuthentication yes
    GSSAPIDelegateCredentials yes
    GSSAPITrustDns yes
```

The OpenSSH version installed in some distributions doesn't have a certain patch
that makes `GSSAPITrustDns` available. In these cases you will need to remove
this keyword from the client configuration file, and add the following to your
`/etc/krb5.conf`:

```
[libdefaults]
    rdns = false
```
