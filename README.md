nixos-update-status
===================

This program is meant to be used as a widget with something like polybar or awesome.

If your system is fully updated, it will simply print out "synced", and if it's outdated, it will print out "unsynced ($)", where the $ represents how many updates (channel bumps) have been missed.

Please keep in mind though that this program is very dumb and will only detect every missed channel bump if you run the program at least once every 4 hours.