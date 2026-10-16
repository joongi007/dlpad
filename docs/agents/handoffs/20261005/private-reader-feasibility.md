# Windows private desktop NVDA feasibility — read-only audit

2026-10-05. Scope: S00-windows-private-reader-feasibility. No download, NVDA launch, native command, project modification, settings change or desktop/input operation performed.

## Result

A portable NVDA process on a unique owned desktop is a plausible **capability experiment**, not accepted actual-reader coverage. Private configuration and --no-sr-flag do not establish host isolation. Existing private UIA results remain UIA evidence only. For actual keyboard reader interaction without changing the user's input desktop, a disposable VM with its own interactive desktop/input/audio boundary is the stronger follow-up. A separate interactive Windows session isolates its input desktop, but shared account registry/profile and audio redirection must be separately controlled; it is not automatically equivalent to a VM.

## Official observations

* Current NV Access online User Guide identifies itself as NVDA 2026.2. It documents -c/--config-path, -f/--log-file, -l/--log-level 12 (input/output), --disable-addons, --minimal, --no-sr-flag and --create-portable-silent/--portable-path. --portable-path is a creation destination, not a running isolation option. Portable creation can overwrite nonempty destinations: any future creation must use fresh owned empty directories.
* --no-sr-flag only suppresses the system screen-reader flag change. In source/nvda.pyw, the non-secure-desktop startup and shutdown still call easeOfAccess.notify(3/2). source/easeOfAccess.py notify first checks HKLM NVDA registration; if registered, it writes the user's Ease of Access temporary registry value and calls SendInput for Win+U/modifier restoration. This path is not conditional on --no-sr-flag. An installed NVDA elsewhere therefore changes the risk of a portable launch. No registry removal or suppression patch is authorized by this audit.
* Startup FindWindow on the thread's desktop can find an existing NVDA and default startup requests its termination, escalating to TerminateProcess if necessary. Never launch on a desktop with an existing reader. --ease-of-access prevents replacement when one is found but is not a complete isolation switch; --quit is inappropriate. The mutex is Local\NVDA_<desktop-name>; systemUtils uses GetThreadDesktop for that name. A unique desktop name supports instance separation, not isolation of all process effects.
* source/winInputHook.py installs low-level keyboard and mouse hooks with thread id zero. Microsoft documents this scope as the calling thread's desktop. Private thread/worker desktop affiliation must therefore be proved. Hooks cannot simply be called global-system hooks, but their absence also cannot be inferred from portable mode.
* Microsoft documents that only the input desktop receives user input. Keeping a new desktop non-input avoids taking the user's screen, but posted owned-window messages do not establish actual low-level reader keyboard interception. UIA focus event/speech generation on a non-input desktop remains untested.
* User Guide supports the No speech synthesizer for Speech Viewer use. source/synthDrivers/silence.py has driver name silence and discards speech output; source/speech/speech.py logs speech sequences with log.io before synthesizer delivery. Thus synthetic speech-sequence log capture with private [speech] synth=silence and log-level 12 is a feasible proposal. It does not prove audible output, every sound is muted, or that non-input focus yields speech. --minimal suppresses startup sounds/UI; do not treat it as a universal audio-isolation switch. No secure mode: it overrides private config and disables logging.

## Proposed argv, not executed

Portable creation, only in a fresh destination: [installer, '--create-portable-silent', '--portable-path', ownedPortable].
Reader candidate: [ownedPortable/nvda.exe, '--config-path', ownedConfig, '--log-file', ownedLog, '--log-level', '12', '--disable-addons', '--minimal', '--no-sr-flag', '--ease-of-access'].
The process must be created suspended, bound to a unique private desktop and an owned Job, then resumed only after host registration/read-only preconditions and desktop identity are checked. This argv is not approval to run it on the current host. Private nvda.ini synthesizer setting requires version-specific validation; no CLI synthesizer selector was found.

## Acceptance for a later capability experiment

Pin actual NVDA executable/version/source and private config bytes. Prove original process creation identity and desktop association for hook/UIA threads; no existing reader on that desktop. Fail closed if registration would trigger notify. Bound synthetic log size/time; collect actual target focus announcements and state/name/read-only changes, preserving native UIA/reader distinction. Assert original desktop unchanged, no shared clipboard/input injection, no unwanted audio, no host configuration changes, natural owned Job empty and private cleanup. If non-input desktop focus does not produce real reader output, retain that failure; do not SwitchDesktop or substitute UIA strings for speech. Keyboard reader command acceptance requires an isolated interactive desktop in a disposable VM/session, not posted-message success.

## Source provenance and limits

Official pages read through web tool, not downloaded or executed. Current master URLs are mutable and are **not a pinned executable/source match**. GitHub commit page exposed f3f2c58df6f13981d27e4ec9bcbecc58a94402b2 (Sep 7 entry); attempted raw files at that commit and GitHub API master query returned tool Internal Error. Do not claim those pinned files were verified. Future executable selection needs exact matching source retrieval.

Primary sources:
- https://download.nvaccess.org/documentation/en/userGuide.html (CLI, portable, No speech)
- https://raw.githubusercontent.com/nvaccess/nvda/master/source/argsParsing.py
- https://raw.githubusercontent.com/nvaccess/nvda/master/source/nvda.pyw (existing instance, mutex, SR flag, notify)
- https://raw.githubusercontent.com/nvaccess/nvda/master/source/easeOfAccess.py (registration guard and SendInput)
- https://raw.githubusercontent.com/nvaccess/nvda/master/source/systemUtils.py (GetThreadDesktop)
- https://raw.githubusercontent.com/nvaccess/nvda/master/source/winInputHook.py
- https://raw.githubusercontent.com/nvaccess/nvda/master/source/synthDrivers/silence.py
- https://raw.githubusercontent.com/nvaccess/nvda/master/source/speech/speech.py
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
- https://learn.microsoft.com/en-us/windows/win32/winstation/desktops
- https://github.com/nvaccess/nvda/commit/f3f2c58df6f13981d27e4ec9bcbecc58a94402b2
