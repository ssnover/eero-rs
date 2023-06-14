# eero-client
Some basic usage of the Eero API in order to grab data on presently connected devices.

To get started: 
```
$ cargo run -- --login EMAIL_ACCT
```
It will take a 6 digit number to verify and authenticate which will be sent to your e-mail.

Then the API will return a cookie string.

Then:
```
$ cargo run -- --cookie "COOKIE"
```

Will show some network information and give you your network url with the numeric id.

```
$ cargo run -- --cookie "COOKIE" --cmd devices
```

Will list all currently detected devices with fields `display_name`, `hostname`, and `connected`. 
This will not show Eero mesh endpoints as they don't have quite the same schema (and they're 
intentionally skipped here).

This code is adapted from this similar python library: https://github.com/343max/eero-client
