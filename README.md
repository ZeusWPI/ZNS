# Zeus (Domain) Name Server

Is implementation of an authoritative DNS server.

It gives all users who have a [Zauth](https://zauth.zeus.gent) account an own domain: `username.users.zeus.gent`.

## General Information

Creating/Updating your DNS records is only possible using dynamic DNS updating (DDNS, rfc2136). 
It's an extension of DNS that lets you update your DNS records using the DNS protocol.

ZNS authenticates these update requests using SIG(0) (rfc2931). 
This is another extension of DNS that defines a signature record. It is appended to the query and contains the signature of the original query and 
some other information like expiration time to prevent replay attacks.

The signature is created with the private key of the signer and validated on the server with the corresponding public key.
ZNS has 2 methods of validating the signature:
- Using your SSH Keys in [Zauth](https://zauth.zeus.gent) 
- Using a [DNSKEY record](https://datatracker.ietf.org/doc/html/rfc4034#section-2)


## User Guide

How to add an `A` record to `<your zauth username>.users.zeus.gent`.

### Step 1

Create an SSH key pair (or use an existing one). Currently, only ED25519 and RSA SSH key types are supported.
Add the public key to your Zauth account.

### Step 2

The (most) painless way for sending DNS update queries is using the `nsupdate` program.
With `nsupdate -k keys`, you can pass it your keys. But `nsupdate` expects your keys to have a certain format, so it won't accept the OPENSSH private key format.
That's why there is a CLI (`zns-cli`) available that converts the OPENSSH private key format and creates `.key` and `.private` files corresponding with your public and private keys.
And with some more info like the update ZONE (`username.users.zeus.gent`), the signing algorithm (ED25519 or RSA), ...

Execute:

```sh
zns-cli --key <path to private ssh key> --username <zauth username>
```

Now you can run `nsupdate -k Kdns.private`.

```
> zone username.users.zeus.gent
> update add username.users.zeus.gent 300 A <ip address>
> send
```

This will add an A record to `username.users.zeus.gent`. 
The message will be signed with the private key, and the server will try to validate by trying to find a valid public SSH key from your Zauth account. Matching the `username` given in the zone.
The default expiration time with `nsupdate` is 5 minutes.

That's it... not that hard, is it?

### Step 3 (Optional)

It is also possible to put your public key in a DNSKEY record instead of Zauth. In the previous step, `zns-cli` also generated a `.key` file. 
This contains a DNSKEY resource record you can add to your zone using `nsupdate`. Now the signature can be validated directly using this record.

It's also possible to directly generate a DNSKEY record key pair using `dnssec-keygen`.

## Server Setup Guide

There are three crates available at the root of the repo.

`zns` is a library which is both used by `zns-cli` and `zns-daemon`.
`zns-daemon` is the server that handles DNS queries.

The following environment variables should be set (or stored in a `.env` file):
```
DATABASE_URL=postgres://zns@localhost/zns
ZAUTH_URL="https://zauth.zeus.gent"
ZONE="users.zeus.gent"
```

Optional: `ZNS_ADDRESS` and `ZNS_PORT`.

After setting `DATABASE_URL`, create the database and run the migrations with `diesel migration run`.

It's quite possible that something is not conform to an RFC, creating an issue is appreciated.