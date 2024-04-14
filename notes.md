m# Pager Notes 2024-04-13

We want to use the lowercase letter phi on the case (it looks cool).
We are going with phiStar.

Using the Qt Py RP2040 board from AdaFruit.

FCC limits you to 30dBm without a license.

## Protocol 0.1

### Sleep-State Alternation

The pager alternates between a state of sleep, receiving, and transmitting on a
consistent schedule.
The sleep state conserves power, the receiving window allows for messages from
the base station, and the transmitting state can be used for reactions.
Even if there is no message to be sent, the base station will send out a ping in
the middle of the sending window, allowing the receiver to synchronize itself
and prevent the base station's sending time from drifting out of the receiver's
receiving window.

### Encryption

Messages will be public-key encrypted, except for a header detailing the intended
recipient in order to prevent needless attempts to decrypt incoming messages.
Additionally, the synchronization pings will not be encrypted, and will have no
contents.

### Parity

Uhh, there is parity of some sort.

### Base Station Registration

Pagers have unique identifiers beginning with a four-digit base-station code that
acts somewhat like an area code for phone numbers.
Base station codes are stored in a registry online in order to prevent collisions.
They are probably just sequential.
