# Task date core

Shared Gregorian date validation and relative input preparation for Plan,
Deadline, Today and Tomorrow. The caller supplies its local calendar day
and lower year bound. Explicit dates and clear pass through unchanged.
Consumers choose which actions accept relative input and enforce their
own output limits.

The library has no clock, filesystem or writer access. Consumers retain
the original action and input as the receipt identity and resolve only
the effective value that goes into the existing local writer. It has no
registry dependencies. Pin the reviewed source revision and lockfile.
