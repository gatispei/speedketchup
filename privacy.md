---
title: SpeedKetchup privacy policy
---

# SpeedKetchup privacy policy

SpeedKetchup does not collect any data. The developer receives nothing from
the app: no results, no usage statistics, no crash reports, no identifiers.
The app has no account, no analytics and no advertising.

## What the app stores

The test results and the settings are stored on your own device only, in
`~/Library/Application Support/SpeedKetchup` (in the app's container when it
is installed from the Mac App Store), or in the file you choose when you run
the command line program. You can delete them at any time.

## What the app connects to

- **Speed test servers.** To measure your connection SpeedKetchup downloads
  the list of public speed test servers and exchanges test data with one of
  them. Like any server you connect to, it sees your IP address. SpeedKetchup
  sends it nothing else.
- **Servers you configure.** Only if you set them up in the settings,
  SpeedKetchup sends every result to your InfluxDB server or MQTT broker, for
  example Home Assistant. Those are your servers, the developer has no access
  to them.
- **Its own web page.** The dashboard is served by the app on your device, at
  `http://127.0.0.1:8288` unless you change it.

## Contact

Questions about privacy: open an issue at
[github.com/gatispei/speedketchup](https://github.com/gatispei/speedketchup/issues).

This policy applies to SpeedKetchup 0.5 and later. Changes are published on
this page.
