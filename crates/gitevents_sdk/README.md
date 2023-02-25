# gitevents sdk

gitevents sdk is the api the user interacts with, it serves as an opininated api
between the gitevents events system and potential handlers the user defines.

All handlers are run in parallel, however, unless the enterprise version is
used, there aren't any guaranteed consistency on delivery from the handlers.

This means that an operation is considered successful even if only a single
handler is executed.
