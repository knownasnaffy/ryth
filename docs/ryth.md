# ryth


## NAME

ryth - Scriptable iwd interface

## SYNOPSIS

<command> [options]

## DESCRIPTION

ryth is a scriptable interface to the iwd wifi backend.
All output is JSON. Designed for use in scripts and automation widgets.
Device and station are selected automatically.

## STATUS

 [--watch] [--help]

Current wifi status (state, signal, powered, etc.)

**Options:**
- **--watch**  
  Stream updates on change
- **-h, --help**  
  Print help

## LIST

 [--watch] [--help]

List discovered networks (cached)

**Options:**
- **--watch**  
  Re-output on each scan cycle
- **-h, --help**  
  Print help

## SCAN

 [--help]

Trigger a scan and return updated network list

**Options:**
- **-h, --help**  
  Print help

## CONNECT

 <SSID> [--password <PASSWORD>] [--help]

Connect to a network by SSID

**Arguments:**
- **<SSID>**  
  The network name (SSID) as a UTF-8 string.

**Options:**
- **--password** <PASSWORD>  
  Passphrase for the network. Omit for open networks.
- **-h, --help**  
  Print help

## DISCONNECT

 [--help]

Disconnect from the current network

**Options:**
- **-h, --help**  
  Print help

## AUTOCONNECT

 <SSID> <STATE> [--help]

Toggle autoconnect for a known network

**Arguments:**
- **<SSID>**  
  The network name (SSID) as a UTF-8 string.
- **<STATE>**  
  Either **on** or **off**.

**Options:**
- **-h, --help**  
  Print help

## FORGET

 <SSID> [--help]

Forget a known network

**Arguments:**
- **<SSID>**  
  The network name (SSID) as a UTF-8 string.

**Options:**
- **-h, --help**  
  Print help

## KNOWN

 [--help]

List known networks

**Options:**
- **-h, --help**  
  Print help

## POWER

 <STATE> [--help]

Toggle wifi power

**Arguments:**
- **<STATE>**  
  Either **on** or **off**.

**Options:**
- **-h, --help**  
  Print help

## OUTPUT

All commands output JSON to stdout. Errors are printed to stderr as:

{"error": "message"}
